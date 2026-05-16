const http = require("http");
const { chromium } = require("playwright");

let browser = null;
let page = null;

async function ensureBrowser() {
  if (!browser) {
    browser = await chromium.launch({
      headless: false,
    });

    page = await browser.newPage();
  }
}

async function readBody(req) {
  return new Promise((resolve) => {
    let body = "";

    req.on("data", (chunk) => {
      body += chunk;
    });

    req.on("end", () => {
      resolve(body);
    });
  });
}

function sendJson(res, status, data) {
  res.writeHead(status, {
    "Content-Type": "application/json",
  });

  res.end(JSON.stringify(data));
}

const server = http.createServer(async (req, res) => {
  try {
    if (req.method === "POST" && req.url === "/open") {
      const body = await readBody(req);
      const { url } = JSON.parse(body);

      if (!url || (!url.startsWith("http://") && !url.startsWith("https://"))) {
        return sendJson(res, 400, {
          error: "Only http and https URLs are allowed.",
        });
      }

      await ensureBrowser();

      await page.goto(url, {
        waitUntil: "domcontentloaded",
        timeout: 30000,
      });

      await page.waitForTimeout(1500);

      const title = await page.title();
      const text = await page.locator("body").innerText();

      const links = await page.$$eval("a[href]", (anchors) =>
        anchors.slice(0, 30).map((a) => ({
          text: a.innerText.trim(),
          href: a.href,
        }))
      );

      return sendJson(res, 200, {
        url,
        title,
        text,
        links,
      });
    }
    
    if (req.method === "GET" && req.url === "/health") {
      return sendJson(res, 200, {
        status: "ok",
      });
    }
    
    if (req.method === "GET" && req.url === "/read") {
      if (!page) {
        return sendJson(res, 400, {
          error: "Browser is not open.",
        });
      }

      const url = page.url();
      const title = await page.title();
      const text = await page.locator("body").innerText();

      const links = await page.$$eval("a[href]", (anchors) =>
        anchors.slice(0, 30).map((a) => ({
          text: a.innerText.trim(),
          href: a.href,
        }))
      );

      return sendJson(res, 200, {
        url,
        title,
        text,
        links,
      });
    }

    if (req.method === "POST" && req.url === "/close") {
      if (browser) {
        await browser.close();
      }

      browser = null;
      page = null;

      return sendJson(res, 200, {
        message: "Browser closed.",
      });
    }

    sendJson(res, 404, {
      error: "Not found.",
    });
  } catch (error) {
    sendJson(res, 500, {
      error: error.message,
    });
  }
});

server.listen(3333, () => {
  console.log("Browser server running on http://localhost:3333");
});