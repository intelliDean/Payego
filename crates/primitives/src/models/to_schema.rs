#[macro_export]
macro_rules! enum_to_schema {
    ($name:ident { $($variant:ident),* $(,)? }) => {
        #[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
        #[serde(rename_all = "lowercase")]
        pub enum $name {
            $(
                #[to_schema(example = stringify!($variant))]
                $variant,
            )*
        }
    };
}
