use crate::util::blocking;
use antidote::Mutex;
use failure::{format_err, Error, Fallible};
use futures::{
    future::{err, Either},
    Future,
};
use lettre::{
    smtp::{
        authentication::Credentials,
        client::net::{ClientTlsParameters, DEFAULT_TLS_PROTOCOLS},
        ClientSecurity, SmtpTransport,
    },
    EmailTransport,
};
use lettre_email::EmailBuilder;
use native_tls::TlsConnector;
use std::sync::Arc;

/// A connection to the mailer.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct Mailer {
    inner: Arc<MailerInner>,
}

impl Mailer {
    /// Connects to an SMTP server.
    pub fn connect(
        host: &str,
        secure: bool,
        user: String,
        pass: String,
        from: String,
    ) -> Fallible<Mailer> {
        let smtp = if secure {
            SmtpTransport::simple_builder(host)?
        } else {
            let mut tls_builder = TlsConnector::builder()?;
            let _ = tls_builder.supported_protocols(DEFAULT_TLS_PROTOCOLS)?;

            let tls_parameters =
                ClientTlsParameters::new(host.to_string(), tls_builder.build().unwrap());

            SmtpTransport::builder(host, ClientSecurity::Opportunistic(tls_parameters))?
        };
        let smtp = smtp
            .credentials(Credentials::new(user.clone(), pass))
            .build();
        Ok(Mailer {
            inner: Arc::new(MailerInner {
                from,
                smtp: Mutex::new(smtp),
            }),
        })
    }

    /// Sends an email.
    pub fn send(
        &self,
        to: &str,
        subject: &str,
        text: &str,
    ) -> impl Future<Item = (), Error = Error> {
        let r = EmailBuilder::new()
            .to(to)
            .from(self.inner.from.as_str())
            .subject(subject)
            .text(text)
            .build();
        let email = match r {
            Ok(email) => email,
            Err(e) => return Either::B(err(e.into())),
        };

        let inner = self.inner.clone();
        Either::A(
            blocking(move || inner.smtp.lock().send(&email))
                .map_err(Error::from)
                .and_then(|resp| {
                    if resp.is_positive() {
                        Ok(())
                    } else {
                        Err(format_err!("Failed to send mail: {:?}", resp))
                    }
                }),
        )
    }
}

#[allow(missing_debug_implementations)]
struct MailerInner {
    from: String,
    smtp: Mutex<SmtpTransport>,
}
