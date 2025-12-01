pub mod sender;

use crate::config::tls::insecure_client_config;
use crate::protocol::message::write_header;
