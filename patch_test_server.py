import re

with open("crates/collab/tests/integration/test_server.rs", "r") as f:
    content = f.read()

config_start = "                kinesis_secret_key: None,"
config_end = "            },"
new_config = """                kinesis_secret_key: None,
                azure_ad_client_id: None,
                azure_ad_client_secret: None,
                azure_ad_tenant_id: None,
                azure_ad_url: None,"""

content = content.replace(config_start, new_config)

with open("crates/collab/tests/integration/test_server.rs", "w") as f:
    f.write(content)
