# IncidentReview: Deployment & Installation Guide

## macOS Installation

### From DMG Bundle (Recommended)

1. Download the latest `.dmg` file from [Releases](https://github.com/saagar210/IncidentReview/releases)
2. Open the DMG file
3. Drag `IncidentReview.app` to the Applications folder
4. Eject the DMG
5. Open Applications folder and launch `IncidentReview`
6. Grant any requested permissions (database access to ~/Library/Application Support/IncidentReview)

### From Source

```bash
git clone https://github.com/saagar210/IncidentReview.git
cd IncidentReview
pnpm install
pnpm tauri build
# DMG will be in src-tauri/target/release/bundle/dmg/
```

## First Run

1. **App Startup**
   - IncidentReview opens to the Workspace section
   - No workspace is active initially

2. **Create a Workspace**
   - Click "Create New Workspace"
   - Provide a workspace name (e.g., "Q1 2025")
   - Select a directory to store the workspace (database will be at `<dir>/incidentreview.sqlite`)
   - App initializes the database and runs 4 migrations automatically
   - Migration status shown in About section

3. **Import Incidents**
   - Navigate to "Jira Import" section
   - Upload a CSV export from Jira (with Summary, Created, Resolved, Impact, Degradation columns)
   - Or navigate to "Slack Import" and paste/upload a Slack transcript
   - Preview the data before import

4. **View Dashboards**
   - Go to "Dashboards" section
   - See incident metrics (MTTD, MTTA, MTTR)
   - View bucketed analytics (by vendor, service, detection source)

5. **Generate Report**
   - Navigate to "Reports" section
   - Click "Generate QIR Report"
   - Markdown report displays incident summary and metrics
   - Copy or export the report

## Optional: Local AI via Ollama

IncidentReview can optionally use local AI models via Ollama for draft generation, theme synthesis, and evidence analysis.

### Enable AI Features

1. **Install Ollama**
   - Download from https://ollama.ai
   - Install and run `ollama serve`
   - This starts Ollama on `127.0.0.1:11434` (localhost only)

2. **Download a Model**
   ```bash
   ollama pull mistral
   # or: ollama pull neural-chat, llama2, etc.
   ```

3. **In IncidentReview**
   - Navigate to "AI" section
   - Click "Health Check" button
   - App checks if Ollama is running
   - If successful, you'll see available models
   - Select a model and generate drafts

### AI Features

- **Draft Generation**: Generate executive summaries, action plans, etc. with AI assistance
- **Evidence Management**: Ingest and index incident evidence (logs, reports, etc.)
- **Citation Enforcement**: All AI-generated content includes citations to evidence
- **Similarity Search**: Find relevant evidence by semantic meaning

### Performance Notes

- **First-time setup**: Downloading a model (e.g., mistral) takes 5-10 minutes
- **Drafting latency**: Generating a draft section takes 10-30 seconds depending on model and evidence size
- **Evidence indexing**: Happens in background; UI remains responsive
- **Ollama requirement**: Ollama must be running on localhost; disconnecting disables AI features (not an error)

### Troubleshooting AI

**"Ollama not detected"**
- Verify Ollama is running: `curl http://127.0.0.1:11434/api/tags`
- Check macOS firewall (should only use localhost, not network)
- Restart IncidentReview after starting Ollama

**"Model not found"**
- Download the model: `ollama pull <model-name>`
- Refresh IncidentReview's model list

**"Embedding failed"**
- Ensure Ollama is still running
- Check available disk space (embeddings are cached)

## Data Storage

### Database Location

```
~/Library/Application Support/IncidentReview/
├── incidentreview.sqlite         (Incidents, artifacts, timeline events)
└── ai/                           (Evidence sources, chunks, embeddings index)
    ├── sources/
    ├── chunks/
    └── index.sqlite
```

### Backup Location

Backups are saved to:
```
~/IncidentReview_Backups/
├── backup_2025-02-12_143022.tar  (Timestamped backup)
└── ...
```

## Backup & Restore

### Creating a Backup

1. Navigate to "Backup & Restore" section
2. Click "Create Backup"
3. Choose output directory
4. IncidentReview creates a timestamped `.tar` file with:
   - Full SQLite database
   - Evidence store
   - Manifest with incident count and metadata

### Restoring from Backup

1. Navigate to "Backup & Restore" section
2. Click "Restore from Backup"
3. Select the backup file (`.tar` format)
4. Choose whether to overwrite existing workspace or merge incidents
5. IncidentReview atomically restores the database

### Important

- Always back up before restoring if you have data you want to keep
- Restore is atomic; either fully succeeds or fully fails
- Backup file should be kept in a safe location (external drive, cloud storage)

## Troubleshooting

### "Database already exists when creating workspace"

- Workspace already created at that location
- Either use "Open Workspace" to connect to it, or
- Delete `incidentreview.sqlite` from the directory and create a new workspace

### "Migrations pending" warning in About section

- IncidentReview automatically applies pending migrations on startup
- If migration fails, check Available Disk Space and Database Integrity
- If stuck, [open an issue](https://github.com/saagar210/IncidentReview/issues)

### "CSV import failed" with "MISSING_REQUIRED_FIELD"

- Ensure CSV has these columns: Summary, Created, Resolved, Impact, Degradation
- Check for spelling and case sensitivity
- Try exporting CSV again from Jira

### "Slack transcript not recognized"

- Verify the file is a Slack export (text or JSON format)
- Check that it contains timestamps and usernames
- See TESTING.md for supported Slack formats

### "Invalid timestamp" warnings in Validation section

- Some timestamps in your data are not RFC3339 format
- IncidentReview preserves the original timestamp for reference
- You can view both the original and parsed timestamps in the incident detail

### App crashes on startup

- Delete the database: `rm ~/Library/Application\ Support/IncidentReview/incidentreview.sqlite`
- Restart IncidentReview
- Create a new workspace
- If crash persists, [open an issue](https://github.com/saagar210/IncidentReview/issues)

## Upgrading

### From v0.x to v1.0.0

- No action needed; existing workspaces are automatically migrated
- Database schema is upgraded via 4 migrations
- No data loss

### Version Information

- Check current version in "About" section
- Upgrade by downloading the latest DMG from Releases

## System Requirements

- **macOS**: 11.0 or later
- **Disk Space**: 2 GB free (more if using Ollama)
- **RAM**: 2 GB minimum
- **Network**: Optional (only needed for Ollama model downloads)

## Support

- **Documentation**: See [README.md](README.md) for features overview
- **Testing**: See [TESTING.md](TESTING.md) for development setup
- **Issues**: [GitHub Issues](https://github.com/saagar210/IncidentReview/issues)
- **Architecture**: See [PLANS.md](PLANS.md) for technical details

## Privacy & Security

- **100% Local-First**: All data stored on your machine
- **No Cloud Services**: No internet required for core features
- **Ollama is Local-Only**: AI features use localhost (127.0.0.1:11434) only
- **Open Source**: Code is publicly auditable

---

**Ready to get started?** See the First Run section above.
