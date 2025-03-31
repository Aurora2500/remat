use std::{
	ffi::CString,
	mem,
	ptr::{null, null_mut},
};

use ffmpeg_sys_next::{
	av_frame_alloc, av_frame_free, av_frame_get_buffer, av_frame_make_writable, av_init_packet,
	av_interleaved_write_frame, av_log_set_level, av_packet_unref, av_write_trailer,
	avcodec_alloc_context3, avcodec_find_encoder, avcodec_free_context, avcodec_open2,
	avcodec_parameters_from_context, avcodec_receive_packet, avcodec_send_frame,
	avdevice_register_all, avformat_alloc_output_context2, avformat_free_context,
	avformat_network_deinit, avformat_network_init, avformat_new_stream, avformat_write_header,
	avio_close, avio_open, memset, AVFormatContext, AVPacket, AVPixelFormat, AVRational,
	AVIO_FLAG_WRITE, AV_LOG_VERBOSE,
};

pub struct Video {}

impl Video {
	pub fn new() -> Self {
		unsafe {
			av_log_set_level(AV_LOG_VERBOSE);
			avdevice_register_all();
			avformat_network_init();
			Self {}
		}
	}
}

impl Drop for Video {
	fn drop(&mut self) {
		unsafe {
			avformat_network_deinit();
		}
	}
}

pub struct Encoder {
	fmt_ctx: *mut AVFormatContext,
}

pub fn test() {
	unsafe {
		let mut format_ctx: *mut AVFormatContext = null_mut();
		avformat_alloc_output_context2(
			&mut format_ctx,
			null(),
			c"mp4".as_ptr(),
			c"output.mp4".as_ptr(),
		);

		let codec = avcodec_find_encoder(ffmpeg_sys_next::AVCodecID::AV_CODEC_ID_H264);
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

		(*codec_ctx).width = 1920;
		(*codec_ctx).height = 1080;
		(*codec_ctx).pix_fmt = AVPixelFormat::AV_PIX_FMT_YUV420P;
		(*codec_ctx).time_base = AVRational { num: 1, den: 25 };
		(*codec_ctx).framerate = AVRational { num: 25, den: 1 };
		(*codec_ctx).gop_size = 10;
		(*codec_ctx).max_b_frames = 1;

		avcodec_open2(codec_ctx, codec, null_mut());
		avcodec_parameters_from_context((*video_stream).codecpar, codec_ctx);

		avio_open(
			&mut (*format_ctx).pb,
			c"output.mp4".as_ptr(),
			AVIO_FLAG_WRITE,
		);
		avformat_write_header(format_ctx, null_mut());

		let mut frame = av_frame_alloc();
		(*frame).width = 1920;
		(*frame).height = 1080;
		(*frame).format = AVPixelFormat::AV_PIX_FMT_YUV420P as i32;
		av_frame_get_buffer(frame, 32);

		for i in 0..1000 {
			av_frame_make_writable(frame);
			memset((*frame).data[0].cast(), i * 2, 1920 * 1080);
			memset((*frame).data[1].cast(), 128, 1920 * 1080 / 4);
			memset((*frame).data[2].cast(), 128, 1920 * 1080 / 4);
			(*frame).pts = i as i64;

			avcodec_send_frame(codec_ctx, frame);
			let mut packet: AVPacket = mem::zeroed();
			av_init_packet(&mut packet);
			while avcodec_receive_packet(codec_ctx, &mut packet) == 0 {
				av_interleaved_write_frame(format_ctx, &mut packet);
				av_packet_unref(&mut packet);
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
