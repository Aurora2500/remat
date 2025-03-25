#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Bindgen does not handle C preprocessor macro functions, so does not handle any of the symbols
// that depend on _IOC, _IO, _IOR, _IOW, _IOWR, etc. Use constant functions to add them.

// C preprocessor macros originally from /usr/include/asm-generic/ioctl.h amended to be Rust
// const functions.

const fn _IOC(dir: u32, type_: char, nr: u32, size: usize) -> u32 {
    (dir << _IOC_DIRSHIFT) | ((type_ as u32) << _IOC_TYPESHIFT) | (nr   << _IOC_NRSHIFT) | ((size as u32) << _IOC_SIZESHIFT)
}

const fn _IO(type_: char, nr: u32) -> u32 { _IOC(_IOC_NONE, type_, nr, 0) }
const fn _IOR(type_: char, nr: u32, size: usize) -> u32 { _IOC(_IOC_READ, type_, nr, size) }
const fn _IOW(type_: char, nr: u32, size: usize) -> u32 { _IOC(_IOC_WRITE, type_, nr, size) }
const fn _IOWR(type_: char, nr: u32, size: usize) -> u32 { _IOC(_IOC_READ|_IOC_WRITE, type_, nr, size) }

// C has a sizeof function, Rust handles this with this const function.

use std::mem::size_of;

// The relevant symbols from /usr/include/linux/videodev2.h

pub const VIDIOC_QUERYCAP:u32 = _IOR('V',  0, size_of::<v4l2_capability>());
pub const VIDIOC_ENUM_FMT:u32 = _IOWR('V',  2, size_of::<v4l2_fmtdesc>());
pub const VIDIOC_G_FMT:u32 = _IOWR('V',  4, size_of::<v4l2_format>());
pub const VIDIOC_S_FMT:u32 = _IOWR('V',  5, size_of::<v4l2_format>());
pub const VIDIOC_REQBUFS:u32 = _IOWR('V',  8, size_of::<v4l2_requestbuffers>());
pub const VIDIOC_QUERYBUF:u32 = _IOWR('V',  9, size_of::<v4l2_buffer>());
pub const VIDIOC_G_FBUF:u32 = _IOR('V', 10, size_of::<v4l2_framebuffer>());
pub const VIDIOC_S_FBUF:u32 = _IOW('V', 11, size_of::<v4l2_framebuffer>());
pub const VIDIOC_OVERLAY:u32 = _IOW('V', 14, size_of::<u32>()); // TODO check C int is Rust u32.
pub const VIDIOC_QBUF:u32 = _IOWR('V', 15, size_of::<v4l2_buffer>());
pub const VIDIOC_EXPBUF:u32 = _IOWR('V', 16, size_of::<v4l2_exportbuffer>());
pub const VIDIOC_DQBUF:u32 = _IOWR('V', 17, size_of::<v4l2_buffer>());
pub const VIDIOC_STREAMON:u32 = _IOW('V', 18, size_of::<u32>()); // TODO check C int is Rust u32.
pub const VIDIOC_STREAMOFF:u32 = _IOW('V', 19, size_of::<u32>()); // TODO check C int is Rust u32.
pub const VIDIOC_G_PARM:u32 = _IOWR('V', 21, size_of::<v4l2_streamparm>());
pub const VIDIOC_S_PARM:u32 = _IOWR('V', 22, size_of::<v4l2_streamparm>());
pub const VIDIOC_G_STD:u32 = _IOR('V', 23, size_of::<v4l2_std_id>());
pub const VIDIOC_S_STD:u32 = _IOW('V', 24, size_of::<v4l2_std_id>());
pub const VIDIOC_ENUMSTD:u32 = _IOWR('V', 25, size_of::<v4l2_standard>());
pub const VIDIOC_ENUMINPUT:u32 = _IOWR('V', 26, size_of::<v4l2_input>());
pub const VIDIOC_G_CTRL:u32 = _IOWR('V', 27, size_of::<v4l2_control>());
pub const VIDIOC_S_CTRL:u32 = _IOWR('V', 28, size_of::<v4l2_control>());
pub const VIDIOC_G_TUNER:u32 = _IOWR('V', 29, size_of::<v4l2_tuner>());
pub const VIDIOC_S_TUNER:u32 = _IOW('V', 30, size_of::<v4l2_tuner>());
pub const VIDIOC_G_AUDIO:u32 = _IOR('V', 33, size_of::<v4l2_audio>());
pub const VIDIOC_S_AUDIO:u32 = _IOW('V', 34, size_of::<v4l2_audio>());
pub const VIDIOC_QUERYCTRL:u32 = _IOWR('V', 36, size_of::<v4l2_queryctrl>());
pub const VIDIOC_QUERYMENU:u32 = _IOWR('V', 37, size_of::<v4l2_querymenu>());
pub const VIDIOC_G_INPUT:u32 = _IOR('V', 38,  size_of::<u32>()); // TODO check C int is Rust u32.
pub const VIDIOC_S_INPUT:u32 = _IOWR('V', 39,  size_of::<u32>()); // TODO check C int is Rust u32.
pub const VIDIOC_G_EDID:u32 = _IOWR('V', 40, size_of::<v4l2_edid>());
pub const VIDIOC_S_EDID:u32 = _IOWR('V', 41, size_of::<v4l2_edid>());
pub const VIDIOC_G_OUTPUT:u32 = _IOR('V', 46,  size_of::<u32>()); // TODO check C int is Rust u32.
pub const VIDIOC_S_OUTPUT:u32 = _IOWR('V', 47,  size_of::<u32>()); // TODO check C int is Rust u32.
pub const VIDIOC_ENUMOUTPUT:u32 = _IOWR('V', 48, size_of::<v4l2_output>());
pub const VIDIOC_G_AUDOUT:u32 = _IOR('V', 49, size_of::<v4l2_audioout>());
pub const VIDIOC_S_AUDOUT:u32 = _IOW('V', 50, size_of::<v4l2_audioout>());
pub const VIDIOC_G_MODULATOR:u32 = _IOWR('V', 54, size_of::<v4l2_modulator>());
pub const VIDIOC_S_MODULATOR:u32 = _IOW('V', 55, size_of::<v4l2_modulator>());
pub const VIDIOC_G_FREQUENCY:u32 = _IOWR('V', 56, size_of::<v4l2_frequency>());
pub const VIDIOC_S_FREQUENCY:u32 = _IOW('V', 57, size_of::<v4l2_frequency>());
pub const VIDIOC_CROPCAP:u32 = _IOWR('V', 58, size_of::<v4l2_cropcap>());
pub const VIDIOC_G_CROP:u32 = _IOWR('V', 59, size_of::<v4l2_crop>());
pub const VIDIOC_S_CROP:u32 = _IOW('V', 60, size_of::<v4l2_crop>());
pub const VIDIOC_G_JPEGCOMP:u32 = _IOR('V', 61, size_of::<v4l2_jpegcompression>());
pub const VIDIOC_S_JPEGCOMP:u32 = _IOW('V', 62, size_of::<v4l2_jpegcompression>());
pub const VIDIOC_QUERYSTD:u32 = _IOR('V', 63, size_of::<v4l2_std_id>());
pub const VIDIOC_TRY_FMT:u32 = _IOWR('V', 64, size_of::<v4l2_format>());
pub const VIDIOC_ENUMAUDIO:u32 = _IOWR('V', 65, size_of::<v4l2_audio>());
pub const VIDIOC_ENUMAUDOUT:u32 = _IOWR('V', 66, size_of::<v4l2_audioout>());
pub const VIDIOC_G_PRIORITY:u32 = _IOR('V', 67, size_of::<u32>()); /* enum v4l2_priority */
pub const VIDIOC_S_PRIORITY:u32 = _IOW('V', 68, size_of::<u32>()); /* enum v4l2_priority */
pub const VIDIOC_G_SLICED_VBI_CAP:u32 = _IOWR('V', 69, size_of::<v4l2_sliced_vbi_cap>());
pub const VIDIOC_LOG_STATUS:u32 = _IO('V', 70);
pub const VIDIOC_G_EXT_CTRLS:u32 = _IOWR('V', 71, size_of::<v4l2_ext_controls>());
pub const VIDIOC_S_EXT_CTRLS:u32 = _IOWR('V', 72, size_of::<v4l2_ext_controls>());
pub const VIDIOC_TRY_EXT_CTRLS:u32 = _IOWR('V', 73, size_of::<v4l2_ext_controls>());
pub const VIDIOC_ENUM_FRAMESIZES:u32 = _IOWR('V', 74, size_of::<v4l2_frmsizeenum>());
pub const VIDIOC_ENUM_FRAMEINTERVALS:u32 = _IOWR('V', 75, size_of::<v4l2_frmivalenum>());
pub const VIDIOC_G_ENC_INDEX:u32 = _IOR('V', 76, size_of::<v4l2_enc_idx>());
pub const VIDIOC_ENCODER_CMD:u32 = _IOWR('V', 77, size_of::<v4l2_encoder_cmd>());
pub const VIDIOC_TRY_ENCODER_CMD:u32 = _IOWR('V', 78, size_of::<v4l2_encoder_cmd>());

/*
 * Experimental, meant for debugging, testing and internal use.
 * Only implemented if CONFIG_VIDEO_ADV_DEBUG is defined.
 * You must be root to use these ioctls. Never use these in applications!
 */
pub const VIDIOC_DBG_S_REGISTER:u32 = _IOW('V', 79, size_of::<v4l2_dbg_register>());
pub const VIDIOC_DBG_G_REGISTER:u32 = _IOWR('V', 80, size_of::<v4l2_dbg_register>());

pub const VIDIOC_S_HW_FREQ_SEEK:u32 = _IOW('V', 82, size_of::<v4l2_hw_freq_seek>());
pub const VIDIOC_S_DV_TIMINGS:u32 = _IOWR('V', 87, size_of::<v4l2_dv_timings>());
pub const VIDIOC_G_DV_TIMINGS:u32 = _IOWR('V', 88, size_of::<v4l2_dv_timings>());
pub const VIDIOC_DQEVENT:u32 = _IOR('V', 89, size_of::<v4l2_event>());
pub const VIDIOC_SUBSCRIBE_EVENT:u32 = _IOW('V', 90, size_of::<v4l2_event_subscription>());
pub const VIDIOC_UNSUBSCRIBE_EVENT:u32 = _IOW('V', 91, size_of::<v4l2_event_subscription>());
pub const VIDIOC_CREATE_BUFS:u32 = _IOWR('V', 92, size_of::<v4l2_create_buffers>());
pub const VIDIOC_PREPARE_BUF:u32 = _IOWR('V', 93, size_of::<v4l2_buffer>());
pub const VIDIOC_G_SELECTION:u32 = _IOWR('V', 94, size_of::<v4l2_selection>());
pub const VIDIOC_S_SELECTION:u32 = _IOWR('V', 95, size_of::<v4l2_selection>());
pub const VIDIOC_DECODER_CMD:u32 = _IOWR('V', 96, size_of::<v4l2_decoder_cmd>());
pub const VIDIOC_TRY_DECODER_CMD:u32 = _IOWR('V', 97, size_of::<v4l2_decoder_cmd>());
pub const VIDIOC_ENUM_DV_TIMINGS:u32 = _IOWR('V', 98, size_of::<v4l2_enum_dv_timings>());
pub const VIDIOC_QUERY_DV_TIMINGS:u32 = _IOR('V', 99, size_of::<v4l2_dv_timings>());
pub const VIDIOC_DV_TIMINGS_CAP:u32 = _IOWR('V', 100, size_of::<v4l2_dv_timings_cap>());
pub const VIDIOC_ENUM_FREQ_BANDS:u32 = _IOWR('V', 101, size_of::<v4l2_frequency_band>());

/*
 * Experimental, meant for debugging, testing and internal use.
 * Never use this in applications!
 */
pub const VIDIOC_DBG_G_CHIP_INFO:u32 = _IOWR('V', 102, size_of::<v4l2_dbg_chip_info>());

pub const VIDIOC_QUERY_EXT_CTRL:u32 = _IOWR('V', 103, size_of::<v4l2_query_ext_ctrl>());

#[cfg(test)]
mod tests {

    use super::*;

    use rstest::rstest;

    // TODO really need to test many more, if not all of the values.

    #[rstest(
        datum, expected,
        case(VIDIOC_QUERYCAP, 2154321408),
        case(VIDIOC_OVERLAY, 1074025998),
        case(VIDIOC_G_PRIORITY, 2147767875),
        case(VIDIOC_S_PRIORITY, 1074026052),
        case(VIDIOC_LOG_STATUS, 22086),
     )]
    fn check_vidioc_symbol_values(datum: u32, expected: u32) {
        assert_eq!(datum, expected);
    }

}
