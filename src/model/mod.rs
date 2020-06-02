macro_rules! define_model_roles {
    (RETRIEVE $obj:ident fn $fn:ident(&self) $(via $via_fn:path)*) => {{
        let field = $obj.$fn().clone();
        $(let field = $via_fn(field);)*
        field.into()
    }};
    (RETRIEVE $obj:ident $field:ident $(via $via_fn:path)*) => {{
        let field = $obj.$field.clone();
        $(let field = $via_fn(field);)*
        field.into()
    }};
    (enum $enum_name:ident for $diesel_model:ident {
     $($role:ident($($retrieval:tt)*): $name:expr),* $(,)?
    }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        enum $enum_name {
            $($role),*
        }

        impl $enum_name {
            #[allow(unused_assignments)]
            fn role_names() -> HashMap<i32, QByteArray> {
                let mut hm = HashMap::new();

                let mut i = 0;
                $(
                    hm.insert(i, $name.into());
                    i += 1;
                )*

                hm
            }

            fn get(&self, obj: &$diesel_model) -> QVariant {
                match self {
                    $(
                        Self::$role => define_model_roles!(RETRIEVE obj $($retrieval)*),
                    )*
                }
            }

            fn from(i: i32) -> Self {
                let rm = [$(Self::$role, )*];
                rm[i as usize]
            }
        }
    };
}

pub mod contact;
pub mod device;
pub mod message;
pub mod session;

pub mod filepicker;
pub mod prompt;

pub use contact::*;
pub use device::*;
pub use message::*;
pub use session::*;

pub use filepicker::*;
pub use prompt::*;
