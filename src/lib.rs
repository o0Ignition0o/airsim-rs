pub mod airsim;
pub mod car;
pub mod controller;
pub mod errors;
mod msgpack;

#[macro_use]
extern crate async_trait;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
