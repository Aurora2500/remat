#[repr(u8)]
pub enum RDTECommand {
	RequestProtocolVersion = 86,
	GetURControlVersion = 118,
	TextMessage = 77,
	DataPackage = 85,
	ControlPackageSetupOutputs = 79,
	ControlPackageSetupInputs = 73,
	ControlPackageStart = 83,
	ControlPackageStop = 80,
}
