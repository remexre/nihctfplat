use crate::util::blocking;
use antidote::Mutex;
use failure::{format_err, Error, Fallible};
use futures::{
    future::{err, Either},
    Future,
};
use lettre::{
    smtp::{authentication::Credentials, SmtpTransport},
    EmailTransport,
};
use lettre_email::EmailBuilder;
use std::sync::Arc;

/// A connections to the mailer.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct Mailer {
    inner: Arc<MailerInner>,
}

impl Mailer {
    /// Connects to an SMTP server.
    pub fn connect(host: &str, user: String, pass: String, from: String) -> Fallible<Mailer> {
        let smtp = SmtpTransport::simple_builder(host)?
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
        html: &str,
        text: &str,
    ) -> impl Future<Item = (), Error = Error> {
        let r = EmailBuilder::new()
            .to(to)
            .from(self.inner.from.as_str())
            .subject(subject)
            .html(html)
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
