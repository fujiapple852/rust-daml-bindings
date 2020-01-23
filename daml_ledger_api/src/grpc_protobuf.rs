#![allow(clippy::all, clippy::pedantic)]
pub mod com {
    pub mod digitalasset {
        pub mod ledger {
            pub mod api {
                pub mod v1 {
                    include!(concat!(env!("OUT_DIR"), "/com.digitalasset.ledger.api.v1.rs"));
                    pub mod testing {
                        include!(concat!(env!("OUT_DIR"), "/com.digitalasset.ledger.api.v1.testing.rs"));
                    }
                    pub mod admin {
                        include!(concat!(env!("OUT_DIR"), "/com.digitalasset.ledger.api.v1.admin.rs"));
                    }
                }
            }
        }
    }
}

pub mod google {
    pub mod rpc {
        include!(concat!(env!("OUT_DIR"), "/google.rpc.rs"));
    }
}
