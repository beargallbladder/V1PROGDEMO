# Quick Deploy to Render

## Option 1: Dashboard Deployment (Recommended)

### Step 1: Create PostgreSQL Database

1. Go to https://dashboard.render.com
2. Click "New +" → "PostgreSQL"
3. Configure:
   - **Name**: `stressor-leads-db`
   - **Database**: `stressor_leads`
   - **User**: `stressor_leads`
   - **Plan**: Free
   - **Region**: Oregon
4. Click "Create Database"
5. **Copy the Internal Database URL** (starts with `postgresql://`)

### Step 2: Create Web Service

1. Go to https://dashboard.render.com
2. Click "New +" → "Web Service"
3. Connect GitHub repository: `beargallbladder/V1PROGDEMO`
4. Configure:
   - **Name**: `stressor-leads-api`
   - **Environment**: `Rust`
   - **Region**: Oregon
   - **Branch**: `main`
   - **Root Directory**: `stressor-leads`
   - **Build Command**: `cargo build --release`
   - **Start Command**: `./target/release/stressor-leads`
   - **Plan**: Free

### Step 3: Set Environment Variables

In the web service settings, add:

- **DATABASE_URL**: (Internal Database URL from Step 1)
- **JWT_SECRET**: Generate with `openssl rand -hex 32`
- **PORT**: `10000`
- **FRONTEND_URL**: (Set after deploying frontend - use `*` for now)

### Step 4: Deploy

Click "Create Web Service" and wait for deployment.

Your API will be at: `https://stressor-leads-api.onrender.com`

## Option 2: Using Render API

You can also use the provided script (though dashboard is easier for first-time setup):

```bash
cd stressor-leads
./deploy-render.sh
```

## Testing

After deployment, test your API:

```bash
curl https://stressor-leads-api.onrender.com/api/health
```

Should return: `OK`

## Next Steps

1. Deploy frontend to Vercel
2. Update `FRONTEND_URL` in Render to your Vercel URL
3. Redeploy backend service

## Troubleshooting

- **Build fails**: Check logs in Render dashboard
- **Database connection fails**: Verify `DATABASE_URL` is the Internal Database URL
- **Service won't start**: Ensure `PORT` is set to `10000`
- **CORS errors**: Set `FRONTEND_URL` to your Vercel domain

