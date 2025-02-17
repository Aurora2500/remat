#pragma once

#include <cstddef>
#include <string>
#include <vector>

struct frame_buffer {
	void *data;
	size_t length;
};

class camera {
private:
	std::string m_device_name;
	int m_fd;
	std::vector<frame_buffer> m_buffers;

	int m_buf_idx;

public:
	camera(std::string const& device_name, int buff_count = 1);
	~camera();

	void take_screenshot(int i);

	void set_exposure(int exposure);
	void set_exposure_auto();
};