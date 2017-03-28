

pub mod loaded_image_protocol;
pub mod device_path_protocol;
pub mod device_path_to_text_protocol;
pub mod simple_text_output_protocol;
pub mod simple_text_input_protocol;
pub mod load_file_protocol;
pub mod load_file2_protocol;
pub mod simple_file_system_protocol;
pub mod file_protocol;

pub use self::loaded_image_protocol::LoadedImageProtocol;
pub use self::device_path_protocol::DevicePathProtocol;
pub use self::device_path_to_text_protocol::DevicePathToTextProtocol;
pub use self::simple_text_output_protocol::SimpleTextOutputProtocol;
pub use self::simple_text_input_protocol::SimpleTextInputProtocol;
pub use self::load_file_protocol::LoadFileProtocol;
pub use self::load_file2_protocol::LoadFile2Protocol;
pub use self::simple_file_system_protocol::SimpleFileSystemProtocol;
pub use self::file_protocol::FileProtocol;

use ::api::types::Guid;

pub trait Protocol {
    fn get_guid() -> Guid;
}
