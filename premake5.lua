-- premake5.lua
workspace "ReMat"
	configurations { "Debug", "Release" }

project "remat"
	kind "ConsoleApp"
	language "C++"
	cppdialect "C++20"
	toolset "clang"
	targetdir "."

	-- PYTHON_INCLUDE = "/usr/include/python3.13"
	-- PYTHON_LIBDIRS = os.outputof("python3-config --ldflags"):gsub("\n", "")

	files { "src/**.hpp", "src/**.cpp" }
	includedirs { "/src" }

	libdirs {}

	PYTHON_LIB = os.outputof("python3-config --libs"):match("-lpython[%d.]+") or "python3"

	links {
		-- "arch", "gf", "kind", "pcp", "sdf", "tf", "usd", "usdGeom", "vt", "work",
		-- PYTHON_LIB
	}

	filter "configurations:Debug"
		defines { "DEBUG" }
		symbols "On"

	filter "configurations:Release"
		defines { "NDEBUG" }
		optimize "On"