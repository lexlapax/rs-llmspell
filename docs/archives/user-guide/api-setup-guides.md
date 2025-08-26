# API Setup Guides for External Tools

This guide provides step-by-step instructions for obtaining and configuring API keys for each external service.

## Table of Contents

1. [Web Search APIs](#web-search-apis)
   - [Google Custom Search](#google-custom-search)
   - [Brave Search](#brave-search)
   - [SerpAPI](#serpapi)
   - [SerperDev](#serperdev)
2. [Email Service APIs](#email-service-apis)
   - [SendGrid](#sendgrid)
   - [AWS SES](#aws-ses)
3. [Database Setup](#database-setup)
   - [PostgreSQL](#postgresql)
   - [MySQL](#mysql)

## Web Search APIs

### Google Custom Search

Google Custom Search provides 100 free queries per day.

#### Step 1: Create a Custom Search Engine

1. Go to [Google Custom Search Engine](https://programmablesearchengine.google.com/controlpanel/create)
2. Click "Add" to create a new search engine
3. In "Sites to search", enter `*.com` (or leave blank to search the entire web)
4. Give your search engine a name
5. Click "Create"
6. Note your Search Engine ID (cx parameter)

#### Step 2: Get API Key

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select existing
3. Enable the Custom Search API:
   - Navigate to "APIs & Services" > "Library"
   - Search for "Custom Search API"
   - Click "Enable"
4. Create credentials:
   - Go to "APIs & Services" > "Credentials"
   - Click "Create Credentials" > "API Key"
   - Copy your API key

#### Step 3: Configure LLMSpell

```bash
export LLMSPELL_API_KEY_GOOGLE="your-api-key"
export GOOGLE_SEARCH_ENGINE_ID="your-search-engine-id"
```

#### Usage Limits
- Free: 100 queries/day
- Paid: $5 per 1000 queries (up to 10k/day)

### Brave Search

Brave Search offers a generous free tier with 2000 queries/month.

#### Step 1: Sign Up

1. Go to [Brave Search API](https://brave.com/search/api/)
2. Click "Get Started"
3. Create an account
4. Verify your email

#### Step 2: Create API Key

1. Log into your Brave account
2. Navigate to the API dashboard
3. Click "Create API Key"
4. Select plan (Free tier available)
5. Copy your API key

#### Step 3: Configure LLMSpell

```bash
export LLMSPELL_API_KEY_BRAVE="your-brave-api-key"
```

#### Usage Limits
- Free: 2000 queries/month
- Basic: $5/month for 20k queries
- Professional: Custom pricing

### SerpAPI

SerpAPI provides real-time Google search results.

#### Step 1: Create Account

1. Visit [SerpAPI](https://serpapi.com/)
2. Click "Sign Up"
3. Verify your email
4. 100 free searches for new accounts

#### Step 2: Get API Key

1. Log into SerpAPI dashboard
2. Navigate to "API Key" section
3. Copy your private API key

#### Step 3: Configure LLMSpell

```bash
export LLMSPELL_API_KEY_SERPAPI="your-serpapi-key"
```

#### Usage Limits
- Free trial: 100 searches
- Hobby: $50/month for 5k searches
- Business: $130/month for 15k searches

### SerperDev

Serper provides Google search results with a simple API.

#### Step 1: Sign Up

1. Go to [SerperDev](https://serper.dev/)
2. Click "Get Started"
3. Sign up with Google or email
4. Get 2500 free credits

#### Step 2: Get API Key

1. Access your dashboard
2. Your API key is displayed on the main page
3. Copy the API key

#### Step 3: Configure LLMSpell

```bash
export LLMSPELL_API_KEY_SERPERDEV="your-serperdev-key"
```

#### Usage Limits
- Free: 2500 searches
- Individual: $10/month for 10k searches
- Team: $50/month for 100k searches

## Email Service APIs

### SendGrid

SendGrid offers 100 emails/day free forever.

#### Step 1: Create Account

1. Go to [SendGrid](https://sendgrid.com/)
2. Click "Start For Free"
3. Fill out the registration form
4. Verify your email address
5. Complete sender identity verification

#### Step 2: Create API Key

1. Log into SendGrid
2. Navigate to Settings > API Keys
3. Click "Create API Key"
4. Choose "Full Access" or "Restricted Access"
5. Name your key (e.g., "LLMSpell Integration")
6. Copy the key immediately (shown only once)

#### Step 3: Verify Sender

1. Go to Settings > Sender Authentication
2. Choose "Single Sender Verification"
3. Add and verify your sender email address
4. Wait for verification email

#### Step 4: Configure LLMSpell

```bash
export LLMSPELL_API_KEY_SENDGRID="SG.your-api-key"
```

#### Usage Limits
- Free: 100 emails/day forever
- Essentials: $19.95/month for 50k emails
- Pro: Custom pricing

### AWS SES

Amazon SES provides cost-effective email sending.

#### Step 1: AWS Account Setup

1. Create [AWS Account](https://aws.amazon.com/) if needed
2. Log into AWS Console
3. Navigate to Amazon SES service
4. Choose your region (e.g., us-east-1)

#### Step 2: Verify Email/Domain

1. In SES Console, go to "Verified identities"
2. Click "Create identity"
3. Choose "Email address" or "Domain"
4. Follow verification steps
5. Check email for verification link

#### Step 3: Create IAM User

1. Go to IAM service
2. Create new user for SES access
3. Attach policy: `AmazonSESFullAccess`
4. Create access key
5. Save Access Key ID and Secret

#### Step 4: Request Production Access

1. By default, SES is in sandbox mode
2. Go to "Account dashboard"
3. Click "Request production access"
4. Fill out the form with use case
5. Wait for approval (usually 24 hours)

#### Step 5: Configure LLMSpell

```bash
export AWS_ACCESS_KEY_ID="your-access-key"
export AWS_SECRET_ACCESS_KEY="your-secret-key"
export AWS_REGION="us-east-1"
```

#### Usage Limits
- First 62,000 emails/month: Free (if from EC2)
- Beyond that: $0.10 per 1000 emails
- Sandbox: 200 emails/day, verified recipients only

## Database Setup

### PostgreSQL

#### Local Development

1. Install PostgreSQL:
   ```bash
   # macOS
   brew install postgresql
   brew services start postgresql
   
   # Ubuntu/Debian
   sudo apt-get install postgresql postgresql-contrib
   sudo systemctl start postgresql
   
   # Windows
   # Download installer from postgresql.org
   ```

2. Create database and user:
   ```bash
   sudo -u postgres psql
   CREATE DATABASE myapp;
   CREATE USER myuser WITH ENCRYPTED PASSWORD 'mypass';
   GRANT ALL PRIVILEGES ON DATABASE myapp TO myuser;
   \q
   ```

3. Configure LLMSpell:
   ```bash
   export DATABASE_URL="postgresql://myuser:mypass@localhost:5432/myapp"
   ```

#### Cloud Providers

**Heroku Postgres**
```bash
# After adding Postgres addon
heroku config:get DATABASE_URL
export DATABASE_URL="<copied-url>"
```

**AWS RDS**
```bash
export DATABASE_URL="postgresql://username:password@endpoint.region.rds.amazonaws.com:5432/dbname"
```

**Google Cloud SQL**
```bash
export DATABASE_URL="postgresql://username:password@/dbname?host=/cloudsql/project:region:instance"
```

### MySQL

#### Local Development

1. Install MySQL:
   ```bash
   # macOS
   brew install mysql
   brew services start mysql
   
   # Ubuntu/Debian
   sudo apt-get install mysql-server
   sudo systemctl start mysql
   
   # Windows
   # Download installer from mysql.com
   ```

2. Secure installation:
   ```bash
   mysql_secure_installation
   ```

3. Create database and user:
   ```sql
   mysql -u root -p
   CREATE DATABASE myapp;
   CREATE USER 'myuser'@'localhost' IDENTIFIED BY 'mypass';
   GRANT ALL PRIVILEGES ON myapp.* TO 'myuser'@'localhost';
   FLUSH PRIVILEGES;
   EXIT;
   ```

4. Configure LLMSpell:
   ```bash
   export DATABASE_URL="mysql://myuser:mypass@localhost:3306/myapp"
   ```

## Configuration Best Practices

### Environment File (.env)

Create a `.env` file for local development:

```bash
# Web Search
LLMSPELL_API_KEY_GOOGLE="your-google-key"
LLMSPELL_API_KEY_BRAVE="your-brave-key"
LLMSPELL_API_KEY_SERPAPI="your-serpapi-key"
LLMSPELL_API_KEY_SERPERDEV="your-serperdev-key"

# Email
LLMSPELL_API_KEY_SENDGRID="your-sendgrid-key"
AWS_ACCESS_KEY_ID="your-aws-key"
AWS_SECRET_ACCESS_KEY="your-aws-secret"
AWS_REGION="us-east-1"

# Database
DATABASE_URL="postgresql://user:pass@localhost/db"
```

### Loading Environment Variables

```bash
# Bash/Zsh
source .env

# Or use a tool like direnv
brew install direnv
echo 'eval "$(direnv hook bash)"' >> ~/.bashrc
```

### Security Tips

1. **Never commit `.env` files** - Add to `.gitignore`
2. **Use different keys for dev/prod** - Separate environments
3. **Rotate keys regularly** - Every 90 days recommended
4. **Limit key permissions** - Use restricted access when possible
5. **Monitor usage** - Set up alerts for unusual activity
6. **Use secret managers** in production:
   - AWS Secrets Manager
   - Google Secret Manager
   - HashiCorp Vault
   - Kubernetes Secrets

### Testing Configuration

Test your API keys are properly configured:

```lua
-- Test script
local function test_api_key(tool_name, params)
    local tool = Tool.get(tool_name)
    local result = tool.execute(params)
    
    if result.success then
        print("✅ " .. tool_name .. " configured correctly")
    else
        print("❌ " .. tool_name .. " error: " .. (result.error.message or "Unknown"))
    end
end

-- Test each service
test_api_key("web_search", {input = "test", provider = "google", max_results = 1})
test_api_key("email-sender", {
    provider = "sendgrid",
    from = "test@example.com",
    to = "test@example.com",
    subject = "Test",
    body = "Test"
})
test_api_key("database-connector", {
    provider = "postgresql",
    connection_string = os.getenv("DATABASE_URL"),
    operation = "query",
    query = "SELECT 1"
})
```

## Troubleshooting

### Common Issues

**"API key not found"**
- Check environment variable spelling
- Ensure variables are exported: `export VAR=value`
- Restart your shell or source the file

**"Invalid API key"**
- Verify key is copied correctly (no extra spaces)
- Check if key needs activation
- Ensure you're using the right key type

**"Rate limit exceeded"**
- Check your plan limits
- Implement caching
- Use multiple API keys
- Add retry logic with backoff

**"Connection refused"**
- Verify service is running (databases)
- Check firewall rules
- Ensure correct host/port
- Test with curl or telnet

### Getting Help

1. Check service-specific documentation
2. Review error messages carefully
3. Enable debug logging: `export LLMSPELL_LOG_LEVEL=debug`
4. Test with minimal examples
5. Check service status pages
6. Contact support with specific error messages

## Next Steps

1. [External Tools Guide](external-tools-guide.md) - Comprehensive tool documentation
2. [Quick Reference](external-tools-quick-reference.md) - Quick command reference
3. [Integration Examples](external-tools-guide.md#integration-examples) - Real-world examples