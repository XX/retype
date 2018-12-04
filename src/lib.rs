extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn replace(attr: TokenStream, item: TokenStream) -> TokenStream {}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
