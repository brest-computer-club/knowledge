pub mod local;

pub trait Walker {
    fn get_root(&self) -> std::io::Result<()>;
}
