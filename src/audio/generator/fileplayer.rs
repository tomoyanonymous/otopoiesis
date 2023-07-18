use crate::audio::Component;
use crate::data::FilePlayerParam;
use crate::parameter::Parameter;
use std::sync::Arc;

use cpal::Sample;
use symphonia::core::codecs::{Decoder, DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error;
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::audio::{SampleBuffer, SignalSpec, Layout};

pub struct FilePlayer<'a> {
    param: Arc<FilePlayerParam>,
    decoder: Option<Box<dyn Decoder + 'a>>,
    track_id: u32,
    format: Box<dyn FormatReader>,
    seek_pos: u64,
    audiobuffer:SampleBuffer<f32>
}
impl<'a> FilePlayer<'a> {
    fn new(param:Arc<FilePlayerParam>)->Self{
        #[cfg(not(target_arch = "wasm32"))]
        let src = std::fs::File::open(param.path.clone()).expect("failed to open media");
        let mut  mss_opts =  MediaSourceStreamOptions::default();
         mss_opts.buffer_len = (param.duration.get() *param.channels.get() as f32) as usize;
         let mss = MediaSourceStream::new(Box::new(src), mss_opts);
         // Create a probe hint using the file's extension. [Optional]
         let mut hint = Hint::new();
         hint.with_extension("mp3");
 
         // Use the default options for metadata and format readers.
         let meta_opts: MetadataOptions = Default::default();
         let fmt_opts: FormatOptions = Default::default();
 
         // Probe the media source.
         let probed = symphonia::default::get_probe()
             .format(&hint, mss, &fmt_opts, &meta_opts)
             .expect("unsupported format");
        let format = probed.format;
        let track = format
        .tracks()
        .iter()
        .find(|t| {
            t.codec_params.codec != CODEC_TYPE_NULL})
        .expect("no supported audio tracks");
    // Use the default options for the decoder.
    let dec_opts: DecoderOptions = Default::default();
    
    // Create a decoder for the track.
    let decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &dec_opts)
        .ok();
    let track_id = track.id;

    let audiobuffer= SampleBuffer::<f32>::new(0,
        SignalSpec::new_with_layout(48000, Layout::Stereo ));
        Self { param, decoder, track_id, format, seek_pos:0, audiobuffer }
    }

    fn get_channels(&self) -> u64 {
        self.param.channels.get()
    }
}
impl<'a> std::fmt::Debug for FilePlayer<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FilePlayer")
            .field("param", &self.param)
            // .field("decoder", ("symphonia decoder(todo)"))
            .finish()
    }
}

impl<'a> Component for FilePlayer<'a> {
    fn get_input_channels(&self) -> u64 {
        self.get_channels()
    }

    fn get_output_channels(&self) -> u64 {
        self.get_channels()
    }

    fn prepare_play(&mut self, info: &crate::audio::PlaybackInfo) {
        self.seek_pos = 0;
        self.audiobuffer= SampleBuffer::<f32>::new(info.frame_per_buffer,
            SignalSpec::new_with_layout(info.sample_rate, Layout::Stereo ));


        #[cfg(target_arch = "wasm32")]
        todo!()
    }

    fn render(&mut self, input: &[f32], output: &mut [f32], info: &crate::audio::PlaybackInfo) {
        // Get the next packet from the media format.
        let packet = match self.format.next_packet() {
            Ok(packet) => packet,
            Err(Error::ResetRequired) => {
                // The track list has been changed. Re-examine it and create a new set of decoders,
                // then restart the decode loop. This is an advanced feature and it is not
                // unreasonable to consider this "the end." As of v0.5.0, the only usage of this is
                // for chained OGG physical streams.
                unimplemented!();
            }
            Err(err) => {
                // A unrecoverable error occurred, halt decoding.
                panic!("{}", err);
            }
        };
        // Consume any new metadata that has been read since the last packet.
        while !self.format.metadata().is_latest() {
            // Pop the old head of the metadata queue.
            self.format.metadata().pop();

            // Consume the new metadata at the head of the metadata queue.
        }

        // If the packet does not belong to the selected track, skip over it.
        if packet.track_id() != self.track_id {
            return;
        }
        let decoder = match self.decoder.as_mut(){
            Some(d) => d,
            None => todo!(),
        };
        // Decode the packet into audio samples.
        match decoder.decode(&packet) {
            Ok(decoded) => {
                // Consume the decoded audio samples (see below)
                self.audiobuffer.copy_interleaved_ref(decoded.clone());
            }
            Err(Error::IoError(_)) => {
                // The packet failed to decode due to an IO error, skip the packet.
                return;
            }
            Err(Error::DecodeError(_)) => {
                // The packet failed to decode due to invalid data, skip the packet.
                return;
            }
            Err(err) => {
                // An unrecoverable error occurred, halt decoding.
                panic!("{}", err);
            }
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn fileload() {
        // FilePlayer::<'static>::
    }
}
