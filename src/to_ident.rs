use convert_case::Casing;

const PERMITTED_CHARS: [char; 62] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
    'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3', '4',
    '5', '6', '7', '8', '9',
];
pub trait ToIdent {
    fn to_ident(&self) -> proc_macro2::Ident;
}

impl<T: ToString> ToIdent for T {
    fn to_ident(&self) -> proc_macro2::Ident {
        let a = self.to_string();
        proc_macro2::Ident::new(
            &a.chars()
                .into_iter()
                .map(|c| if PERMITTED_CHARS.contains(&c) { c } else { '_' })
                .collect::<String>()
                .from_case(convert_case::Case::Snake)
                .to_case(convert_case::Case::ScreamingSnake),
            proc_macro2::Span::call_site(),
        )
    }
}
