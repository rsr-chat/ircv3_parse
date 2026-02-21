pub mod de;
pub mod ser;

mod builder;

pub use builder::MessageBuilder;

use crate::compat::{Debug, Display, FmtResult, Formatter};

use crate::components::{Commands, Params, Source, Tags};
use crate::scanner::Scanner;

/// A parsed IRC message.
#[derive(Clone, Copy)]
pub struct Message<'a> {
    input: &'a str,
    scanner: Scanner,
}

impl<'a> Message<'a> {
    #[inline]
    pub fn new(input: &'a str, scanner: Scanner) -> Self {
        Self { input, scanner }
    }

    /// Returns [`Tags`] if present.
    #[inline]
    pub fn tags(&self) -> Option<Tags<'a>> {
        if self.scanner.has_tags() {
            Some(Tags::new(self.scanner.tags_span.extract(self.input)))
        } else {
            None
        }
    }

    /// Returns [`Source`] if present.
    #[inline]
    pub fn source(&self) -> Option<Source<'a>> {
        if self.scanner.has_source() {
            Some(Source::parse(self.scanner.source_span.extract(self.input)))
        } else {
            None
        }
    }

    /// Returns [`Commands`].
    #[inline]
    pub fn command(&self) -> Commands<'a> {
        Commands::from(self.scanner.command_span.extract(self.input))
    }

    /// Returns [`Params`].
    #[inline]
    pub fn params(&self) -> Params<'a> {
        if self.scanner.has_trailing() {
            let start_pos = self.scanner.params_span.start as usize;
            let end_pos = self.scanner.trailing_span.end as usize;
            let input = &self.input[start_pos..end_pos];

            Params::new(
                input,
                self.scanner.params_span.extract(self.input),
                Some(self.scanner.trailing_span.extract(self.input)),
            )
        } else {
            let input = self.scanner.params_span.extract(self.input);
            Params::new(input, input, None)
        }
    }

    /// Fetch the raw input `&str` backing this `Message`.
    pub fn input_raw(&self) -> &str {
        self.input
    }
}

impl Display for Message<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(self.input)
    }
}

impl Debug for Message<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct(stringify!(Message))
            .field("tags", &self.tags())
            .field("source", &self.source())
            .field("command", &self.command())
            .field("params", &self.params())
            .finish()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Message<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let tags = self.tags();
        let source = self.source();
        let params = self.params();
        let has_params = !params.middles.is_empty() || params.trailing.is_some();

        let field_count =
            1 + tags.is_some() as usize + source.is_some() as usize + has_params as usize;

        let mut state = serializer.serialize_struct("Message", field_count)?;

        if let Some(tags) = tags {
            state.serialize_field("tags", &tags)?;
        }

        if let Some(source) = source {
            state.serialize_field("source", &source)?;
        }

        state.serialize_field("command", &self.command())?;

        if has_params {
            state.serialize_field("params", &params)?;
        }

        state.end()
    }
}
