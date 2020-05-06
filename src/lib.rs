#[macro_use]
extern crate enum_primitive_derive;
extern crate num_traits;

pub mod types;
pub mod ctx;
pub mod bindings;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
