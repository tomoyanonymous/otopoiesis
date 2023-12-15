use ariadne::{Label, Report, ReportKind, Source};
use std::path;

pub trait ReportableError: std::error::Error {
    /// message is used for reporting verbose message for ariadne.
    ///
    fn get_span(&self) -> std::ops::Range<usize>;
    fn get_message(&self) -> String {
        self.to_string()
    }
    /// label is used for indicating error with the specific position for ariadne.
    fn get_label(&self) -> String {
        self.to_string()
    }
}

pub fn report_to(
    src: &str,
    srcpath: path::PathBuf,
    errs: &[Box<dyn ReportableError>],
    mut w: impl std::io::Write,
) {
    let path = srcpath.to_str().unwrap_or_default();
    for e in errs {
        let span = e.get_span();
        // let a_span = (src.source(), span);
        let builder = Report::build(ReportKind::Error, "test", 4)
            .with_message(e.get_message().as_str())
            .with_label(Label::new((path, span.clone())).with_message(e.get_label().as_str()))
            .finish();
        let cache_key = (path, Source::from(src));
        builder.write(cache_key, &mut w).unwrap()
    }
}

pub fn report(src: &str, srcpath: path::PathBuf, errs: &[Box<dyn ReportableError>]) {
    report_to(src, srcpath, errs, std::io::stderr())
}

pub fn report_to_string(
    src: &str,
    srcpath: path::PathBuf,
    errs: &[Box<dyn ReportableError>],
) -> String {
    let mut buf = Vec::<u8>::new();
    report_to(src, srcpath, errs, &mut buf);
    String::from(String::from_utf8_lossy(buf.as_slice()))
}

pub fn dump_to_string(errs: &[Box<dyn ReportableError>]) -> String {
    let mut res = String::new();
    for e in errs {
        res += e.get_message().as_str();
    }
    res
}
