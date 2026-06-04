# automatically generated release

to automatically curl the latest setup script, use this command:

```bash
curl -Ls $(curl -s https://api.github.com/repos/gold007-dev/remote-access-server/releases/latest | jq '.assets.[-1].browser_download_url' | sed 's/"//gm')
```
