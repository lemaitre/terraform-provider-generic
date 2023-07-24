---
# generated by https://github.com/hashicorp/terraform-plugin-docs
page_title: "cmd_ssh_sensitive_file Resource - cmd"
subcategory: ""
description: |-
  Reads a remote file
---

# cmd_ssh_sensitive_file (Resource)

Reads a remote file



<!-- schema generated by tfplugindocs -->
## Schema

### Required

- `path` (String) Remote path to the file

### Optional

- `connect` (Block List, Max: 1) Connection configuration (see [below for nested schema](#nestedblock--connect))
- `content` (String, Sensitive) Content of the remote file
- `content_base64` (String, Sensitive) Content of the remote file encoded in base64
- `content_source` (String, Sensitive) Content of the remote file from a local file
- `keep` (Boolean) Content of the remote file
- `mode` (String) Content of the remote file
- `overwrite` (Boolean) Content of the remote file

### Read-Only

- `id` (String) Id of the fiel resource
- `md5` (String) MD5 fingerprint of the file (hex)
- `sha1` (String) SHA1 fingerprint of the file (hex)
- `sha256` (String) SHA256 fingerprint of the file (hex)
- `sha256_base64` (String) SHA256 fingerprint of the file (base64)
- `sha512` (String) SHA512 fingerprint of the file (hex)
- `sha512_base64` (String) SHA512 fingerprint of the file (base64)

<a id="nestedblock--connect"></a>
### Nested Schema for `connect`

Required:

- `host` (String) Hostname to connect to

Optional:

- `key` (String) Key
- `keyfile` (String) Filename of the key
- `password` (String) Password or passphrase
- `port` (Number) Port to connect to
- `user` (String) User to connect with