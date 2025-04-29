use std::{
	ffi::CString,
	mem,
	ptr::{self, null, null_mut, write_bytes},
	time::Instant,
};

use ffmpeg_sys_next::{
	av_frame_alloc, av_frame_free, av_frame_get_buffer, av_frame_make_writable,
	av_interleaved_write_frame, av_log_set_level, av_packet_alloc, av_packet_unref,
	av_write_trailer, avcodec_alloc_context3, avcodec_find_encoder, avcodec_free_context,
	avcodec_open2, avcodec_parameters_from_context, avcodec_receive_packet, avcodec_send_frame,
	avformat_alloc_output_context2, avformat_free_context, avformat_network_deinit,
	avformat_network_init, avformat_new_stream, avformat_write_header, avio_close, avio_open,
	memset, AVCodec, AVCodecContext, AVColorRange, AVFormatContext, AVFrame, AVPacket,
	AVPixelFormat, AVRational, AVStream, AVIO_FLAG_WRITE, AV_LOG_WARNING,
};

pub struct VideoContext {}

impl VideoContext {
	pub fn new() -> Self {
		unsafe {
			av_log_set_level(AV_LOG_WARNING);
			avformat_network_init();
			Self {}
		}
	}
}

impl Drop for VideoContext {
	fn drop(&mut self) {
		unsafe {
			avformat_network_deinit();
		}
	}
}

#[must_use]
pub struct Encoder {
	format_ctx: *mut AVFormatContext,
	codec_ctx: *mut AVCodecContext,
	frame: *mut AVFrame,
}

impl Encoder {
	pub fn new() -> Self {
		unsafe {
			let time_base = AVRational { num: 1, den: 25 };
			let framerate = AVRational { num: 25, den: 1 };

			let mut format_ctx = null_mut();
			avformat_alloc_output_context2(&mut format_ctx, null(), c"matroska".as_ptr(), null());

			let codec = avcodec_find_encoder(ffmpeg_sys_next::AVCodecID::AV_CODEC_ID_FFV1);
			if codec.is_null() {
				panic!("Failed getting codec");
			}
			let video_stream = avformat_new_stream(format_ctx, codec);
			if video_stream.is_null() {
				panic!("Failed getting video stream");
			}
			let codec_ctx = avcodec_alloc_context3(codec);
			if codec_ctx.is_null() {
				panic!("Failed getting codec context");
			}

			(*video_stream).time_base = AVRational { num: 1, den: 25 };
			(*codec_ctx).width = 1920;
			(*codec_ctx).height = 1080;
			(*codec_ctx).pix_fmt = AVPixelFormat::AV_PIX_FMT_YUV444P;
			(*codec_ctx).color_range = AVColorRange::AVCOL_RANGE_JPEG;
			(*codec_ctx).time_base = time_base;
			(*codec_ctx).framerate = framerate;
			(*codec_ctx).gop_size = 10;
			(*codec_ctx).max_b_frames = 1;
			(*video_stream).time_base = time_base;

			avcodec_open2(codec_ctx, codec, null_mut());
			avcodec_parameters_from_context((*video_stream).codecpar, codec_ctx);

			avio_open(
				&mut (*format_ctx).pb,
				c"output.mkv".as_ptr(),
				AVIO_FLAG_WRITE,
			);
			avformat_write_header(format_ctx, null_mut());

			let frame = av_frame_alloc();
			(*frame).width = 1920;
			(*frame).height = 1080;
			(*frame).format = AVPixelFormat::AV_PIX_FMT_YUV444P as i32;
			(*frame).color_range = AVColorRange::AVCOL_RANGE_JPEG;
			av_frame_get_buffer(frame, 32);

			Self {
				format_ctx,
				codec_ctx,
				frame,
			}
		}
	}

	pub fn encode(&mut self, pts: i64, y: &[u8], cb: &[u8], cr: &[u8]) {
		unsafe {
			av_frame_make_writable(self.frame);
			ptr::copy_nonoverlapping(y.as_ptr(), (*self.frame).data[0], y.len());
			ptr::copy_nonoverlapping(cb.as_ptr(), (*self.frame).data[0], cb.len());
			ptr::copy_nonoverlapping(cr.as_ptr(), (*self.frame).data[0], cr.len());
			// let image = create_image(i as u8);
			// ptr::copy_nonoverlapping(image, (*frame).data[0], 1920 * 1080 * 3);
			(*self.frame).pts = pts * 40;
			(*self.frame).time_base = AVRational { num: 1, den: 25 };

			avcodec_send_frame(self.codec_ctx, self.frame);
			let packet = av_packet_alloc();
			while avcodec_receive_packet(self.codec_ctx, packet) == 0 {
				av_interleaved_write_frame(self.format_ctx, packet);
				av_packet_unref(packet);
			}
		}
	}

	pub fn finish(self) {
		let Self {
			format_ctx,
			mut codec_ctx,
			mut frame,
		} = self;

		unsafe {
			avcodec_send_frame(codec_ctx, null_mut());
			let mut packet: AVPacket = mem::zeroed();
			while avcodec_receive_packet(codec_ctx, &mut packet) == 0 {
				av_interleaved_write_frame(format_ctx, &mut packet);
				av_packet_unref(&mut packet);
			}

			av_write_trailer(format_ctx);
			avcodec_free_context(&mut codec_ctx);
			av_frame_free(&mut frame);
			avio_close((*format_ctx).pb);
			avformat_free_context(format_ctx);
		}
	}
}

pub fn test() {
	unsafe {
		let time_base = AVRational { num: 1, den: 25 };
		let framerate = AVRational { num: 25, den: 1 };

		let mut format_ctx = null_mut();
		avformat_alloc_output_context2(&mut format_ctx, null(), c"matroska".as_ptr(), null());

		let codec = avcodec_find_encoder(ffmpeg_sys_next::AVCodecID::AV_CODEC_ID_FFV1);
		if codec.is_null() {
			panic!("Failed getting codec");
		}
		let video_stream = avformat_new_stream(format_ctx, codec);
		if video_stream.is_null() {
			panic!("Failed getting video stream");
		}
		let mut codec_ctx = avcodec_alloc_context3(codec);
		if codec_ctx.is_null() {
			panic!("Failed getting codec context");
		}

		(*video_stream).time_base = AVRational { num: 1, den: 25 };
		(*codec_ctx).width = 1920;
		(*codec_ctx).height = 1080;
		(*codec_ctx).pix_fmt = AVPixelFormat::AV_PIX_FMT_YUV420P;
		(*codec_ctx).color_range = AVColorRange::AVCOL_RANGE_JPEG;
		(*codec_ctx).time_base = time_base;
		(*codec_ctx).framerate = framerate;
		(*codec_ctx).gop_size = 10;
		(*codec_ctx).max_b_frames = 1;
		(*video_stream).time_base = time_base;

		avcodec_open2(codec_ctx, codec, null_mut());
		avcodec_parameters_from_context((*video_stream).codecpar, codec_ctx);

		avio_open(
			&mut (*format_ctx).pb,
			c"output.mkv".as_ptr(),
			AVIO_FLAG_WRITE,
		);
		avformat_write_header(format_ctx, null_mut());

		let mut frame = av_frame_alloc();
		(*frame).width = 1920;
		(*frame).height = 1080;
		(*frame).format = AVPixelFormat::AV_PIX_FMT_YUV420P as i32;
		av_frame_get_buffer(frame, 32);

		for i in 0..1000i32 {
			av_frame_make_writable(frame);
			write_bytes((*frame).data[0].cast::<u8>(), i as u8, 1920 * 1080);
			memset((*frame).data[1].cast(), 128, 1920 * 1080 / 4);
			memset((*frame).data[2].cast(), 128, 1920 * 1080 / 4);
			// let image = create_image(i as u8);
			// ptr::copy_nonoverlapping(image, (*frame).data[0], 1920 * 1080 * 3);
			(*frame).pts = i as i64 * 40;
			(*frame).time_base = AVRational { num: 1, den: 25 };

			avcodec_send_frame(codec_ctx, frame);
			let packet = av_packet_alloc();
			while avcodec_receive_packet(codec_ctx, packet) == 0 {
				av_interleaved_write_frame(format_ctx, packet);
				av_packet_unref(packet);
			}
		}

		avcodec_send_frame(codec_ctx, null_mut());
		let mut packet: AVPacket = mem::zeroed();
		while avcodec_receive_packet(codec_ctx, &mut packet) == 0 {
			av_interleaved_write_frame(format_ctx, &mut packet);
			av_packet_unref(&mut packet);
		}

		av_write_trailer(format_ctx);
		avcodec_free_context(&mut codec_ctx);
		av_frame_free(&mut frame);
		avio_close((*format_ctx).pb);
		avformat_free_context(format_ctx);
	}
}
