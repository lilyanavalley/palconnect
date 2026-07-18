// 
// PalConnect - A Discord bot for PalWorld server monitoring
// Copyright (C) 2025  Lily Ana Valley <hi@lilyvalley.dev> <https://lilyvalley.dev>
//
// This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General 
// Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) 
// any later version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied 
// warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more
// details.
// 
// You should have received a copy of the GNU Affero General Public License along with this program.  If not, see
// <https://www.gnu.org/licenses/>.
// 

// * Values don't have to be lowercase; we lowercase them in the code for comparison.
const SENSITIVE_FIELDS: &[&str] = &[
    "adminpassword", "admin_password", "password", "passwd", "pwd",
    "secret", "token", "key", "api", "api_key", "apikey",
    "auth", "authorization", "credential", "cred", "rcon",
    "credential_master_key",
];

/// Recursively sanitize sensitive data from JSON values.
/// Any field whose name matches a known sensitive pattern is replaced with a redaction marker.
pub fn sanitize_sensitive_data(mut value: serde_json::Value) -> serde_json::Value {
    match &mut value {
        serde_json::Value::Object(map) => {
            for (key, val) in map.iter_mut() {
                let key_lc = key.to_lowercase();
                let is_sensitive = SENSITIVE_FIELDS.iter().any(|&field| {
                    key_lc == field
                        || key_lc.starts_with(field)
                        || key_lc.starts_with(&format!("{}_", field))
                        || key_lc.starts_with(&format!("{}-", field))
                        || key_lc.ends_with(&format!("_{}", field))
                        || key_lc.ends_with(&format!("-{}", field))
                        || key_lc.ends_with(field)
                });
                if is_sensitive {
                    *val = serde_json::Value::String("▷ REDACTED ◁".to_string());
                } else {
                    *val = sanitize_sensitive_data(std::mem::take(val));
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr.iter_mut() {
                *item = sanitize_sensitive_data(std::mem::take(item));
            }
        }
        _ => {}
    }
    value
}
