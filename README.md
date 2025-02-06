# Setup Instructions

## Windows Users
If you are using Windows, open **PowerShell as Administrator** and run the following commands to allow the necessary ports through the firewall:

```powershell
# Allow port 8000 (HTTP server)
New-NetFirewallRule -DisplayName "Rust HTTP Server" -Direction Inbound -LocalPort 8000 -Protocol TCP -Action Allow

# Allow port 8080 (WebSocket server)
New-NetFirewallRule -DisplayName "Rust WebSocket" -Direction Inbound -LocalPort 8080 -Protocol TCP -Action Allow
```

## Linux Users
If you are using Linux, run the following commands in the terminal to allow the necessary ports through the firewall:

```bash
# Allow ports through firewall (Linux)
sudo ufw allow 8000/tcp
sudo ufw allow 8080/tcp
```

## Accessing the Webpage
After setting up the firewall rules, you can access the webpage by opening your browser and entering:

```
localhost:8000/index.html
```