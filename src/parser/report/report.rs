use super::PError;

pub type PResult<T> = Result<T, PReport>;

pub struct PReport {
    errors: Vec<PError>,
}

impl From<PError> for PReport {
    fn from(error: PError) -> Self {
        Self {
            errors: vec![error],
        }
    }
}

impl PReport {
    pub fn empty() -> Self {
        Self { errors: vec![] }
    }

    pub fn append(&mut self, mut report: PReport) {
        self.errors.append(&mut report.errors);
    }

    pub fn errors(&self) -> impl Iterator<Item = &PError> {
        self.errors.iter()
    }
}
