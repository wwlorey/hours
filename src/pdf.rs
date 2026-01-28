use std::path::Path;

use anyhow::Result;

use crate::config::LicensureConfig;
use crate::data::model::HoursData;

pub fn generate_report(
    _data: &HoursData,
    _config: &LicensureConfig,
    _output_path: &Path,
) -> Result<()> {
    anyhow::bail!("PDF export is not yet implemented. Coming in Phase 7.")
}
