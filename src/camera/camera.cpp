#include "camera.hpp"
#include <iostream>
#include <fstream>
#include <fcntl.h>
#include <cstring>
#include <unistd.h>
#include <format>

#include <linux/videodev2.h>
#include <sys/ioctl.h>
#include <sys/mman.h>

camera::camera(std::string const&device_name, int buff_count)
	: m_device_name(device_name)
{
	m_fd = open(device_name.c_str(), O_RDWR);
	if (m_fd < 0) {
		std::cerr << "Couldn't open device" << std::endl;
	}

	v4l2_format format = {0};
	format.type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
	format.fmt.pix.width = 640;
	format.fmt.pix.height = 480;
	format.fmt.pix.pixelformat = V4L2_PIX_FMT_MJPEG;
	format.fmt.pix.field = V4L2_FIELD_NONE;

	if (ioctl(m_fd, VIDIOC_S_FMT, &format) < 0) {
		std::cerr << "VIDIOC_S_FMT error" << std::endl;
	}

	v4l2_requestbuffers req = {0};
	req.count = buff_count;
	req.type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
	req.memory = V4L2_MEMORY_MMAP;
	if (ioctl(m_fd, VIDIOC_REQBUFS, &req) < 0) {
		std::cerr << "VIDIOC_REQBUFS error" << std::endl;
	}

	for (int i = 0; i < buff_count; i++) {
		auto& buf = m_buffers.emplace_back();
		v4l2_buffer b = {0};
		b.type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
		b.memory = V4L2_MEMORY_MMAP;
		b.index = i;
		if (ioctl(m_fd, VIDIOC_QUERYBUF, &b) < 0) {
			std::cerr << "VIDIOC_QUERYBUF error" << std::endl;
		}
		buf.length = b.length;
		buf.data = mmap(NULL, b.length,
			PROT_READ | PROT_WRITE,
			MAP_SHARED, m_fd, b.m.offset);
	}

	int ty = V4L2_BUF_TYPE_VIDEO_CAPTURE;
	if (ioctl(m_fd, VIDIOC_STREAMON, &ty) < 0) {
		std::cerr << "VIDIOC_STREAMON" << std::endl;
	}
}

camera::~camera() {
	int ty = V4L2_CAP_VIDEO_CAPTURE;
	ioctl(m_fd, VIDIOC_STREAMOFF, &ty);
	for (auto& buf: m_buffers) {
		munmap(buf.data, buf.length);
	}
	close(m_fd);
}

void camera::take_screenshot(int i) {
	v4l2_buffer b = {0};
	b.type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
	b.memory = V4L2_MEMORY_MMAP;
	b.index = m_buf_idx++;
	if (m_buf_idx <= m_buffers.size()) {
		m_buf_idx = 0;
	}

	if (ioctl(m_fd, VIDIOC_QBUF, &b) < 0) {
		std::cerr << "VIDIOC_QBUF" << std::endl;
	}
	if (ioctl(m_fd, VIDIOC_DQBUF, &b) < 0) {
		std::cerr << "VIDIOC_DQBUF" << std::endl;
	}
	std::ofstream file(std::format("image{0}.jpeg", i), std::ios::binary);
	auto& buff = m_buffers[b.index];
	file.write((char*)buff.data, b.bytesused);
	file.close();
	std::cout << "Frame saved!" << std::endl;
}

void camera::set_exposure(int exposure)
{
	v4l2_control ctrl = {0};
	ctrl.id = V4L2_CID_EXPOSURE_AUTO;
	ctrl.value = 1; // manual mode
	ioctl(m_fd, VIDIOC_S_CTRL, &ctrl);

	ctrl.id = V4L2_CID_EXPOSURE_ABSOLUTE;
	ctrl.value = exposure;
	ioctl(m_fd, VIDIOC_S_CTRL, &ctrl);
}

void camera::set_exposure_auto()
{
	v4l2_control ctrl = {0};
	ctrl.id = V4L2_CID_EXPOSURE_AUTO;
	ctrl.value = 3; // auto mode
	ioctl(m_fd, VIDIOC_S_CTRL, &ctrl);
}
