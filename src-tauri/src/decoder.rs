use ffmpeg_next as ffmpeg;
use ffmpeg_next::format::{input, Pixel};
use ffmpeg_next::media::Type;
use ffmpeg_next::software::scaling::{context::Context, flag::Flags};
use ffmpeg_next::util::frame::video::Video;
use std::path::Path;

pub struct Decoder {
    input_ctx: ffmpeg::format::context::Input,
    video_stream_index: usize,
    decoder: ffmpeg::decoder::Video,
    scaler: Context,
    target_width: u32,
    target_height: u32,
}

impl Decoder {
    pub fn new(path: &Path, low_quality: bool) -> anyhow::Result<Self> {
        ffmpeg::init()?;

        let input_ctx = input(&path)?;
        let stream = input_ctx
            .streams()
            .best(Type::Video)
            .ok_or_else(|| anyhow::anyhow!("No video stream found"))?;

        let video_stream_index = stream.index();
        let mut context_decoder =
            ffmpeg::codec::context::Context::from_parameters(stream.parameters())?;

        // Note: Discard optimization disabled temporarily due to ffmpeg-next version mismatch
        // if low_quality {
        //     context_decoder.set_discard(ffmpeg::codec::Discard::NonRef);
        // }

        let mut decoder = context_decoder.decoder().video()?;

        let width = decoder.width();
        let height = decoder.height();

        let (target_width, target_height) = if low_quality {
            (width / 4, height / 4)
        } else {
            (width, height)
        };

        let scaler = Context::get(
            decoder.format(),
            width,
            height,
            Pixel::RGBA,
            target_width,
            target_height,
            Flags::BILINEAR,
        )?;

        Ok(Self {
            input_ctx,
            video_stream_index,
            decoder,
            scaler,
            target_width,
            target_height,
        })
    }

    pub fn decode_next_frame(&mut self) -> anyhow::Result<Option<Vec<u8>>> {
        let mut frame = Video::empty();

        for (stream, packet) in self.input_ctx.packets() {
            if stream.index() == self.video_stream_index {
                self.decoder.send_packet(&packet)?;
                if self.decoder.receive_frame(&mut frame).is_ok() {
                    let mut scaled_frame = Video::empty();
                    self.scaler.run(&frame, &mut scaled_frame)?;

                    // Return the raw RGBA data
                    return Ok(Some(scaled_frame.data(0).to_vec()));
                }
            }
        }

        Ok(None)
    }

    pub fn get_dimensions(&self) -> (u32, u32) {
        (self.target_width, self.target_height)
    }
}
