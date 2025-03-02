pub mod get_account_info;
pub mod msg_delivery;
pub mod upload;

pub mod v1 {
    include!("./generated/service.ourchat.v1.rs");
}

pub mod download {
    pub mod v1 {
        include!("./generated/service.ourchat.download.v1.rs");
    }
}

pub mod session {
    pub mod new_session {
        pub mod v1 {
            include!("./generated/service.ourchat.session.new_session.v1.rs");
        }
    }

    pub mod accept_session {
        pub mod v1 {
            include!("./generated/service.ourchat.session.accept_session.v1.rs");
        }
    }

    pub mod invite_session {
        pub mod v1 {
            include!("./generated/service.ourchat.session.invite_session.v1.rs");
        }
    }

    pub mod get_session_info {
        pub mod v1 {
            include!("./generated/service.ourchat.session.get_session_info.v1.rs");
        }
    }

    pub mod set_session_info {
        pub mod v1 {
            include!("./generated/service.ourchat.session.set_session_info.v1.rs");
        }
    }

    pub mod set_role {
        pub mod v1 {
            include!("./generated/service.ourchat.session.set_role.v1.rs");
        }
    }

    pub mod add_role {
        pub mod v1 {
            include!("./generated/service.ourchat.session.add_role.v1.rs");
        }
    }
}

pub mod set_account_info {
    pub mod v1 {
        include!("./generated/service.ourchat.set_account_info.v1.rs");
    }
}

pub mod unregister {
    pub mod v1 {
        include!("./generated/service.ourchat.unregister.v1.rs");
    }
}
