with open("crates/collab/src/lib.rs", "r") as f:
    lines = f.readlines()

new_lines = []
for line in lines:
    new_lines.append(line)
    if "pub supermaven_admin_api_key: Option<Arc<str>>," in line:
        new_lines.append("    pub azure_ad_client_id: Option<String>,\n")
        new_lines.append("    pub azure_ad_client_secret: Option<String>,\n")
        new_lines.append("    pub azure_ad_tenant_id: Option<String>,\n")
        new_lines.append("    pub azure_ad_url: Option<String>,\n")
    if "supermaven_admin_api_key: None," in line:
        new_lines.append("            azure_ad_client_id: None,\n")
        new_lines.append("            azure_ad_client_secret: None,\n")
        new_lines.append("            azure_ad_tenant_id: None,\n")
        new_lines.append("            azure_ad_url: None,\n")

with open("crates/collab/src/lib.rs", "w") as f:
    f.writelines(new_lines)
