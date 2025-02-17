#include <iostream>
#include "camera/camera.hpp"

// #include <pxr/usd/usd/stage.h>
// #include <pxr/usd/usdGeom/sphere.h>

int main() {
	camera cam("/dev/video0", 5);
	cam.set_exposure(1000);
	for (int i = 0; i < 20; i++) {
		cam.take_screenshot(i);
	}

	// pxr::UsdStageRefPtr stage = pxr::UsdStage::CreateNew("example.usda");
	// pxr::UsdGeomSphere sphere = pxr::UsdGeomSphere::Define(stage, pxr::SdfPath("/Sphere"));
	// // sphere.GetRadiusAttr().Set(2.0);
	// stage->GetRootLayer()->Save();
	return 0;
}