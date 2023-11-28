use crate::app::filemanager::{self, FileManager};

use crate::audio::Component;
use crate::data::{ConversionError, FilePlayerParam};
use crate::parameter::Parameter;
use crate::script::{Expr, Value};
use std::io::ErrorKind;

use symphonia::core::audio::{Layout, SampleBuffer, SignalSpec};
use symphonia::core::codecs::{Decoder, DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::{MediaSource, MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::{Hint, ProbeResult};
use symphonia::core::units::Time;

pub struct FilePlayer {
    param: FilePlayerParam,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    channels: u64,
    format: Box<dyn FormatReader>,
    audiobuffer: SampleBuffer<f32>,
    ringbuf: ringbuf::HeapRb<f32>,
    is_finished_playing: bool,
}

type DecoderSet = (Box<dyn Decoder>, ProbeResult, u32);

fn get_default_decoder(path: impl ToString) -> Result<DecoderSet, Box<dyn std::error::Error>> {
    let flmgr = filemanager::get_global_file_manager();
    let src = flmgr.open_file_stream(path).expect("failed to open file");
    let ms: Box<dyn MediaSource> = Box::new(src);
    let mss_opts = MediaSourceStreamOptions::default();
    let mss = MediaSourceStream::new(ms, mss_opts);
    let hint = Hint::new();
    //  hint.with_extension("mp3");
    // Use the default options for metadata and format readers.
    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();

    // Probe the media source.
    let probed = symphonia::default::get_probe().format(&hint, mss, &fmt_opts, &meta_opts)?;

    let format = probed.format.as_ref();
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or(Error::Unsupported("no supported audio tracks"))?;
    let id = track.id;
    // Use the default options for the decoder.
    let dec_opts: DecoderOptions = Default::default();
    // Create a decoder for the track.
    let decoder = symphonia::default::get_codecs().make(&track.codec_params, &dec_opts)?;

    Ok((decoder, probed, id))
}

impl FilePlayer {
    pub fn new(param: FilePlayerParam) -> Self {
        let buf_len = MediaSourceStreamOptions::default().buffer_len;
        let path_str = param.path.try_lock().expect("failed to lock");
        let (decoder, probed, track_id) = get_default_decoder(path_str).expect("decoder not found");
        let channels = decoder.codec_params().channels.unwrap().count() as u64;
        let max_frames = decoder.codec_params().max_frames_per_packet.unwrap();
        let audiobuffer = SampleBuffer::<f32>::new(
            max_frames,
            SignalSpec::new_with_layout(48000, Layout::Stereo),
        );

        let ringbuf = ringbuf::HeapRb::new(buf_len);
        Self {
            param,
            decoder,
            track_id,
            channels,
            format: probed.format,
            audiobuffer,
            ringbuf,
            is_finished_playing: false,
        }
    }
    pub fn is_finished_playing(&self) -> bool {
        self.is_finished_playing
    }
    pub fn get_channels(&self) -> u64 {
        self.channels
    }
}
impl std::fmt::Debug for FilePlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FilePlayer")
            .field("param", &self.param)
            // .field("decoder", ("symphonia decoder(todo)"))
            .finish()
    }
}

impl TryFrom<&Value> for FilePlayer {
    type Error = ConversionError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Closure(
                _ids,
                env,
                box Expr::App(box Expr::Var(fname), args),
                // _name,
                // _type,
            ) if fname == "fileplayer" && args.len() == 3 => {
                let pathv = args
                    .get(0)
                    .and_then(|a| a.eval(env.clone(), &None, &mut None).ok())
                    .ok_or(ConversionError {})?;
                let start_sec = args
                    .get(1)
                    .and_then(|a| a.eval(env.clone(), &None, &mut None).ok())
                    .ok_or(ConversionError {})?;
                let duration = args
                    .get(2)
                    .and_then(|a| a.eval(env.clone(), &None, &mut None).ok())
                    .ok_or(ConversionError {})?;
                if let (
                    Value::String(path),
                    Value::Parameter(start_sec),
                    Value::Parameter(duration),
                ) = (pathv, start_sec, duration)
                {
                    let param = FilePlayerParam {
                        path,
                        start_sec,
                        duration,
                    };
                    Ok(Self::new(param))
                } else {
                    Err(Self::Error {})
                }
            }
            _ => Err(Self::Error {}),
        }
    }
}

impl Component for FilePlayer {
    fn get_input_channels(&self) -> u64 {
        self.get_channels()
    }

    fn get_output_channels(&self) -> u64 {
        self.get_channels()
    }

    fn prepare_play(&mut self, _info: &crate::audio::PlaybackInfo) {
        let start_sec = self.param.start_sec.get();
        let time_sec = start_sec.floor() as u64;
        let time_frac = start_sec.fract() as f64;
        self.format
            .seek(
                SeekMode::Accurate,
                SeekTo::Time {
                    time: Time::new(time_sec, time_frac),
                    track_id: Some(0),
                },
            )
            .unwrap_or_else(|_| panic!("failed to seek position {}", start_sec));
        self.is_finished_playing = false;
    }

    fn render(&mut self, _input: &[f32], output: &mut [f32], _info: &crate::audio::PlaybackInfo) {
        output.fill(0.0);
        // Get the next packet from the media format.
        let (mut prod, mut cons) = self.ringbuf.split_ref();
        let mut read_count = 0;
        let mut finished_loop = false;
        while !finished_loop {
            let reached_eof = if cons.len() < output.len() {
                match self.format.next_packet() {
                    Ok(packet) => {
                        // Consume any new metadata that has been read since the last packet.
                        while !self.format.metadata().is_latest() {
                            // Pop the old head of the metadata queue.
                            self.format.metadata().pop();

                            // Consume the new metadata at the head of the metadata queue.

                            // If the packet does not belong to the selected track, skip over it.
                            if packet.track_id() != self.track_id {
                                return;
                            }
                        }
                        // Decode the packet into audio samples.
                        let res = self.decoder.decode(&packet).map(|decoded| {
                            // Consume the decoded audio samples (see below)
                            self.audiobuffer.copy_interleaved_ref(decoded.clone());
                            let _nsamples = prod.push_slice(self.audiobuffer.samples());
                            // println!(
                            //     "frames:{}, timestamp:{}, n_samples: {}",
                            //     decoded.frames(),
                            //     packet.ts(),
                            //     _nsamples
                            // );
                        });
                        res.map(|()| false)
                    }
                    Err(Error::ResetRequired) => {
                        // The track list has been changed. Re-examine it and create a new set of decoders,
                        // then restart the decode loop. This is an advanced feature and it is not
                        // unreasonable to consider this "the end." As of v0.5.0, the only usage of this is
                        // for chained OGG physical streams.
                        unimplemented!();
                    }
                    Err(Error::IoError(err)) if err.kind() == ErrorKind::UnexpectedEof => Ok(true),
                    Err(err) => Err(err),
                }
            } else {
                // println!("ring buffer has remaining ");
                Ok(false)
            };
            self.is_finished_playing = reached_eof.map_or(false, |res| res);

            let read_len = cons.len().min(output.len());
            // dbg!(read_count, cons.len(),output.len());
            let next_read = (read_count + read_len).min(output.len());

            let output_buf = &mut output[read_count..next_read];
            read_count += read_len;
            cons.pop_slice(output_buf);

            finished_loop = self.is_finished_playing || read_count > output.len() - 1;
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{audio::PlaybackInfo, data};

    use super::*;
    fn read_prep() -> (FilePlayer, PlaybackInfo, usize) {
        let (param, len_samples) = data::FilePlayerParam::new_test_file();
        let player = FilePlayer::new(param);
        let info = PlaybackInfo {
            sample_rate: 48000,
            current_time: 0,
            frame_per_buffer: 256,
            channels: 2,
        };
        (player, info, len_samples)
    }
    #[test]
    fn render_long_buffer() {
        let (mut player, info, len_samples) = read_prep();
        player.prepare_play(&info);
        let mut output_buf = vec![0.0f32; len_samples * 2 + 1];
        let input_buf = vec![0.0f32; 512];
        player.render(&input_buf, output_buf.as_mut_slice(), &info);
        assert!(player.is_finished_playing());
    }
    #[test]
    fn render_long_buffer_fail() {
        let (mut player, info, len_samples) = read_prep();
        player.prepare_play(&info);
        let mut output_buf = vec![0.0f32; len_samples * 2];
        let input_buf = vec![0.0f32; 512];
        player.render(&input_buf, output_buf.as_mut_slice(), &info);
        assert!(!player.is_finished_playing());
    }
    #[test]
    fn render_small_chunks() {
        let (mut player, mut info, len_samples) = read_prep();
        player.prepare_play(&info);
        let samples = 512;
        let mut output_buf = vec![0.0f32; samples * 2];
        let input_buf = vec![0.0f32; samples * 2];
        let read_count_max = (len_samples as f32 / samples as f32).floor() as usize;
        for _i in 0..read_count_max {
            player.render(&input_buf, output_buf.as_mut_slice(), &info);
            info.current_time += samples;
            println!("test read {}", info.current_time);
        }
        assert!(!player.is_finished_playing());
        player.render(&input_buf, output_buf.as_mut_slice(), &info);
        assert!(player.is_finished_playing());
    }
}
