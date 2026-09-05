# Documentation & GitHub Pages Rules for Magento Doctor

These rules govern the maintenance, visual presentation, and technical accuracy of the Magento Doctor showcase website (`docs/`) and technical documentation.

## 1. Visual & Technical Design Standards
- **Premium Aesthetics**: The website must maintain a dark-mode first, cyber-forensics aesthetic utilizing custom CSS (no generic framework defaults). Use curated color palettes (deep obsidian `#080b11`, cyan `#00f0ff`, emerald `#00ff9d`, amber `#ffb800`).
- **Zero Heavy Runtime Dependencies**: The showcase site must remain lightweight, fast-loading, and self-contained with pure semantic HTML5, Vanilla CSS, and Vanilla JavaScript.
- **Interactive First**: Keep terminal examples, rule search, copy-to-clipboard interactions, and version matrix toggles reactive and immediately responsive.

## 2. Technical Accuracy & Capabilities Coverage
Any updates to the website must accurately reflect the CLI's capabilities:
- **Core Philosophy**: Always emphasize `Collect → Model → Correlate → Diagnose` and explain how multi-signal correlation outperforms disconnected checklist warnings.
- **Rule Taxonomy**: Maintain synchronization with the rule catalog (`MD-CRON-*`, `MD-PLG-*`, `MD-PERF-*`, `MD-DB-*`, `MD-CACHE-*`, `MD-ENV-*`, `MD-SEC-*`, `MD-DI-*`).
- **Stack Compatibility**: Keep the supported version matrix updated with:
  - Magento Open Source & Adobe Commerce 2.4.4 through 2.4.9
  - PHP 8.1 through 8.5 (certified for 2.4.9)
  - MariaDB 10.4, 10.6, 10.11, 11.x (11.0–11.4 LTS)
  - MySQL 8.0, 8.4 LTS, 9.x (latest)
  - OpenSearch 1.2 through 3.0 (actively supported)
  - Redis 6.2 through 8.0 & Valkey 7.2, 8.0 (actively supported drop-in replacement)

## 3. Security & Zero-Exposure Policy
- **Zero Secrets**: Never display actual passwords, crypt keys, or tokens in web examples or screenshots. Always show `SecretValue::Present` or redacted representations.
- **Safe Operations**: Clearly document safe read-only query boundaries and the `--offline` flag.
