# Time Helm

- Open-source 
- Persistent-world 
- Massively-multiplayer online 
- Sandbox 
- Social simulation game


# Game Mechanics
- One hour in-game lasts one minute real-time.
- For the sake of "divide-by-60" symmetry, one game-year consists of 360 game-days.
- Game world and game rules are reality-centric (i.e. no supernatural or sci-fi elements)
- The ability of the player to control their character(s) is dependent on the avatar's emotional state, like `The Sims` but perhaps to a greater extent
- Players mostly (only?) control their characters indirectly, specifying a schedule and if-then conditions


# Graphics
- Low-poly 3D graphics
- Graphical realism and animation are a secondary concern 



# Technical

- **Frontend**: Three.js (browser-based 3D graphics)
- **Backend**: Rust server with Axum
- **Authentication**: X (Twitter) OAuth 2.0
- **Real-time Communication**: WebSockets
- **Deployment**: fly.io
- **Domain**: time-helm.com


# Planned Development Path
- Start with a person in a house, with modern amenities that are puchased from "outside world"
OR
- Start with a person in a cave?


# Required Research
- How long does it take to make basic things?
- How productive are ancient farming techniques?


# Other Ideas
- Once sufficiently realistic, can be used as platform for modelling and tracing human history in detail, and assessing different hypotheses


# Preliminary Notes and Lists

##  Human Activities

See https://github.com/julerex/omniscim/activities.md


##  Human Health Condition
- Hunger
- Thirst
- Disease
- Disability
- Age
- Exhaustion
- Sleepiness
- Agitation
- Anxiety
- Depression
- Mental Illness
- Sadness
- Loneliness
- Jealousy
- Anger
- 


## Human Properties
- Genetics


## Household Objects
- Hygiene
- Cleaning
- Clothes
- Electronics
- Stationary
- Furniture
- Small appliances
- Kitchen appliances
- Kitchen utensils
- Cutlery
- Crockery
- 

## Science Simulation

See (materials.md)[https://github.com/julerex/omniscim/materials.md]

- Physics
- Chemistry
- Biology
  - Human Biology
  - Food Science
  - Epidemiology
  - Botany

- Medicine
- Geology
- Meteorology
- Hydrology
- Metallurgy
- Chemical Engineering
- Civil Engineering
- Mechanical Engineering


# Setup & Development

## Prerequisites

- Rust (latest stable)
- Node.js 18+ and npm
- PostgreSQL 14+ (local development)
- Twitter Developer Account (for OAuth credentials)
- fly.io account

## Local Development

### 1. Set up PostgreSQL Database

**Using Docker:**
```bash
docker run --name timehelm-db -e POSTGRES_PASSWORD=password -e POSTGRES_DB=timehelm -p 5432:5432 -d postgres:15
```

**Or install PostgreSQL locally:**
```bash
# Ubuntu/Debian
sudo apt-get install postgresql postgresql-contrib

# macOS
brew install postgresql
brew services start postgresql

# Create database
createdb timehelm
```

### 2. Set up Twitter OAuth

1. Go to [Twitter Developer Portal](https://developer.twitter.com/)
2. Create a new app
3. Enable OAuth 2.0
4. Set callback URL to `http://localhost:8080/auth/twitter/callback`
5. Copy your Client ID and Client Secret

### 3. Configure Environment

Create a `.env` file in the project root:

```bash
# Server
PORT=8080
BASE_URL=http://localhost:8080

# Database (adjust for your setup)
DATABASE_URL=postgresql://postgres:password@localhost:5432/timehelm

# Twitter OAuth
TWITTER_CLIENT_ID=your_client_id
TWITTER_CLIENT_SECRET=your_client_secret
```

**Note:** The database migrations will run automatically when the server starts.

### 4. Build and Run

**Terminal 1 - Backend:**
```bash
cd server
cargo run
```

**Terminal 2 - Frontend:**
```bash
cd client
npm install
npm run dev
```

The game will be available at `http://localhost:5173` (Vite dev server proxies API calls to the backend).

## Deployment to fly.io

### 1. Install flyctl

```bash
curl -L https://fly.io/install.sh | sh
```

### 2. Login to fly.io

```bash
fly auth login
```

### 3. Create the app

```bash
fly apps create timehelm
```

### 4. Create PostgreSQL Database on fly.io

```bash
fly postgres create --name timehelm-db
fly postgres attach --app timehelm timehelm-db
```

This will automatically set the `DATABASE_URL` secret for your app.

### 5. Set environment variables

```bash
fly secrets set TWITTER_CLIENT_ID=your_client_id
fly secrets set TWITTER_CLIENT_SECRET=your_client_secret
fly secrets set BASE_URL=https://time-helm.com
```

**Note:** The `DATABASE_URL` is automatically set when you attach the PostgreSQL database.

### 6. Configure DNS with Cloudflare

#### Create the certificate in fly.io

```bash
fly certs create time-helm.com
```

This will output DNS records that need to be added to Cloudflare.

2. **Add DNS records in Cloudflare:**
   - Log into your Cloudflare dashboard
   - Select your domain `time-helm.com`
   - Go to **DNS** → **Records**
   - Add the records provided by fly.io:
     - **Type**: `A` or `AAAA` (depending on what fly.io provides)
     - **Name**: `@` (or `time-helm.com`)
     - **IPv4/IPv6 address**: The value from fly.io
     - **Proxy status**: Turn OFF the proxy (gray cloud) - This is important! Fly.io needs direct DNS resolution for SSL certificate validation
     - **TTL**: Auto
   
   - If fly.io provides a CNAME record:
     - **Type**: `CNAME`
     - **Name**: `@` or the subdomain
     - **Target**: The value from fly.io
     - **Proxy status**: Turn OFF (gray cloud)
     - **TTL**: Auto

3. **Important Cloudflare Settings:**
   - **SSL/TLS mode**: Set to "Full" or "Full (strict)" in SSL/TLS settings
   - **Always Use HTTPS**: Enable this in SSL/TLS → Edge Certificates
   - **Proxy status**: Keep DNS records in "DNS only" mode (gray cloud) until certificate is verified

4. **Wait for certificate verification:**
   ```bash
   fly certs show time-helm.com
   ```
   Wait until you see "Issued" status. This can take a few minutes.

5. **After certificate is issued:**
   - You can optionally enable Cloudflare proxy (orange cloud) for DDoS protection, but be aware:
     - Fly.io handles SSL termination, so Cloudflare proxy is optional
     - If you enable proxy, ensure SSL/TLS mode is set to "Full" or "Full (strict)"
     - Some WebSocket connections may work better with proxy disabled

### 7. Build and deploy

First, build the frontend:
```bash
cd client
npm install
npm run build
```

Then deploy:
```bash
fly deploy
```

The app will be available at `https://time-helm.com`

## Project Structure

```
timehelm/
├── server/          # Rust backend
│   ├── src/
│   │   ├── main.rs      # Server entry point
│   │   ├── auth.rs      # Twitter OAuth handling
│   │   ├── game.rs      # Game state management
│   │   └── websocket.rs # WebSocket handlers
│   └── Cargo.toml
├── client/          # Three.js frontend
│   ├── src/
│   │   ├── main.js         # Entry point
│   │   ├── auth-client.js  # Auth API client
│   │   └── game-client.js  # Game client & Three.js scene
│   ├── index.html
│   └── package.json
├── Dockerfile       # Container build config
├── fly.toml         # fly.io configuration
└── vite.config.js   # Vite dev server config
```

# Preliminary Links

## General
- https://mrminimal.gitlab.io/2018/07/26/godot-dedicated-server-tutorial.html
- https://docs.godotengine.org/en/stable/getting_started/workflow/export/exporting_for_dedicated_servers.html
- https://developer.valvesoftware.com/wiki/Dedicated_Servers_List
- https://www.gtxgaming.co.uk/
- https://gotm.io/

## Github Links
- https://github.com/riperiperi/FreeSO - A full reimplementation of The Sims Online, using Monogame.

## Open Source Assets
- Check itch.io
- https://sketchfab.com/zaxel/collections/human-male-anatomy-reference
- https://github.com/makehumancommunity/makehuman
