use crate::audio::Component;
use crate::data::FilePlayerParam;
use crate::parameter::Parameter;
use std::io::ErrorKind;
use std::sync::Arc;

use symphonia::core::audio::{Layout, SampleBuffer, SignalSpec};
use symphonia::core::codecs::{Decoder, DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error;
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::{Hint, ProbeResult};

pub struct FilePlayer {
    param: Arc<FilePlayerParam>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    format: Box<dyn FormatReader>,
    seek_pos: u64,
    audiobuffer: SampleBuffer<f32>,
    ringbuf: ringbuf::HeapRb<f32>,
    is_finished_playing: bool,
}

fn get_default_decoder(
    mss: MediaSourceStream,
) -> Result<(Box<dyn Decoder>, ProbeResult, u32), symphonia::core::errors::Error> {
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
    pub fn new(param: Arc<FilePlayerParam>) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let src = std::fs::File::open(param.path.clone()).expect("failed to open media");
        #[cfg(target_arch = "wasm32")]
        let src = todo!();

        let mss_opts = MediaSourceStreamOptions::default();
        let buf_len = mss_opts.buffer_len;
        let mss = MediaSourceStream::new(Box::new(src), mss_opts);

        let (decoder, probed, track_id) = get_default_decoder(mss).expect("decoder not found");

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
            format: probed.format,
            seek_pos: 0,
            audiobuffer,
            ringbuf,
            is_finished_playing: false,
        }
    }
    pub fn is_finished_playing(&self) -> bool {
        self.is_finished_playing
    }
    pub fn get_channels(&self) -> u64 {
        self.param.channels.get()
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

impl Component for FilePlayer {
    fn get_input_channels(&self) -> u64 {
        self.get_channels()
    }

    fn get_output_channels(&self) -> u64 {
        self.get_channels()
    }

    fn prepare_play(&mut self, _info: &crate::audio::PlaybackInfo) {
        self.seek_pos = 0;
        self.is_finished_playing = false;
        // self.audiobuffer = SampleBuffer::<f32>::new(
        //     info.frame_per_buffer,
        //     SignalSpec::new_with_layout(info.sample_rate, Layout::Stereo),
        // );
    }

    fn render(&mut self, _input: &[f32], output: &mut [f32], _info: &crate::audio::PlaybackInfo) {
        output.fill(0.0);
        // Get the next packet from the media format.
        let (mut prod, mut cons) = self.ringbuf.split_ref();
        let mut read_count = 0;
        let mut is_reading = true;
        while is_reading {
            let res_play_finished = match self.format.next_packet() {
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
                        // output.copy_from_slice(self.audiobuffer.samples());
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
            };
            match res_play_finished {
                Ok(finished) => {
                    self.is_finished_playing = finished;
                }
                Err(e) => {
                    // A unrecoverable error occurred, halt decoding.
                    panic!("{:?}", e);
                }
            }

            let next_read = read_count + cons.len();
            let end_point = if next_read > output.len() {
                output.len()
            } else {
                next_read
            };
            let output_buf = &mut output[read_count..end_point];
            read_count += cons.len();
            cons.pop_slice(output_buf);
            is_reading = !self.is_finished_playing && read_count < output.len() - 1;
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{audio::PlaybackInfo, data};
    use std::sync::Arc;

    use super::*;
    fn read_prep() -> (FilePlayer, PlaybackInfo, usize) {
        let (param, len_samples) = data::FilePlayerParam::new_test_file();
        let player = FilePlayer::new(Arc::new(param));
        let info = PlaybackInfo {
            sample_rate: 48000,
            current_time: 0,
            frame_per_buffer: 256,
            channels: 2,
        };
        (player, info, len_samples)
    }
    #[test]
    fn fileload() {
        let (mut player, mut info, len_samples) = read_prep();
        player.prepare_play(&info);
        let mut output_buf = vec![0.0f32; 512];
        let input_buf = vec![0.0f32; 512];
        let read_count_max = (len_samples as f32 / 256.0).floor() as usize;
        for _i in 0..read_count_max {
            player.render(&input_buf, output_buf.as_mut_slice(), &info);
            info.current_time += 256;
            // println!("{}", info.current_time);
        }
        assert!(!player.is_finished_playing());
        player.render(&input_buf, output_buf.as_mut_slice(), &info);
        assert!(player.is_finished_playing());
    }
}
