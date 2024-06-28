const CORS_PROXY = "https://api.allorigins.win/raw?url=";

const rssFeedList = document.getElementById("fd");
const infoDialog = document.getElementById("info-dialog");
const opmlForm = document.getElementById("opml-form");
const opmlInput = document.getElementById("opml-url");
const dialog = document.getElementById("info-dialog");
const showButton = document.getElementById("show-button");
const closeButton = document.getElementById("close-button");
let allPosts = [];

function timeSince(date) {
  const seconds = Math.floor((new Date() - date) / 1000);
  let interval = seconds / (365 * 24 * 60 * 60);

  if (interval > 1) {
    return Math.floor(interval) + " years ago";
  }
  interval = seconds / (30 * 24 * 60 * 60);
  if (interval > 1) {
    return Math.floor(interval) + " months ago";
  }
  interval = seconds / (24 * 60 * 60);
  if (interval > 1) {
    return Math.floor(interval) + " days ago";
  }
  interval = seconds / (60 * 60);
  if (interval > 1) {
    return Math.floor(interval) + " hours ago";
  }
  interval = seconds / 60;
  if (interval > 1) {
    return Math.floor(interval) + " minutes ago";
  }
  return Math.floor(seconds) + " seconds ago";
}

function getDomain(url) {
  return new URL(url).hostname;
}

async function fetchWithCORSRetry(url) {
  try {
    return await fetch(url);
  } catch (error) {
    return await fetch(CORS_PROXY + url);
  }
}

async function fetchOPML(url) {
  const response = await fetchWithCORSRetry(url);
  if (!response.ok) {
    throw new Error("Failed to fetch OPML file");
  }
  const text = await response.text();
  return new window.DOMParser().parseFromString(text, "text/xml");
}

async function fetchFeed(url) {
  const response = await fetchWithCORSRetry(url);
  if (!response.ok) {
    throw new Error("Failed to fetch RSS feed");
  }
  const text = await response.text();
  return new window.DOMParser().parseFromString(text, "application/xml");
}

function parseOPML(opmlDoc) {
  const outlines = opmlDoc.querySelectorAll('outline[type="rss"]');
  return Array.from(outlines).map((outline) => outline.getAttribute("xmlUrl"));
}

function parseFeed(feedDoc, sourceUrl) {
  const isAtom = feedDoc.documentElement.nodeName === "feed";
  if (isAtom) {
    return parseAtom(feedDoc, sourceUrl);
  } else {
    return parseRSS(feedDoc, sourceUrl);
  }
}

function parseRSS(rssDoc, sourceUrl) {
  const items = rssDoc.querySelectorAll("item");
  return Array.from(items).map((item) => ({
    title: item.querySelector("title").textContent,
    link: item.querySelector("link").textContent,
    pubDate: new Date(item.querySelector("pubDate").textContent),
    source: getDomain(sourceUrl),
  }));
}

function parseAtom(atomDoc, sourceUrl) {
  const entries = atomDoc.querySelectorAll("entry");
  return Array.from(entries).map((entry) => ({
    title: entry.querySelector("title").textContent,
    link: entry.querySelector("link").getAttribute("href"),
    pubDate: new Date(
      entry.querySelector("updated").textContent ||
        entry.querySelector("published").textContent,
    ),
    source: getDomain(sourceUrl),
  }));
}

function sortPostsByDate(posts) {
  return posts.sort((a, b) => b.pubDate - a.pubDate);
}

function displayPosts(posts) {
  rssFeedList.innerHTML = "";
  posts.forEach((post) => {
    const listItem = document.createElement("li");
    const link = document.createElement("a");
    link.href = post.link;
    link.textContent = post.title;
    const pubDate = document.createElement("span");
    pubDate.className = "entry-sub";
    pubDate.innerHTML = `<br>published ${
      timeSince(post.pubDate)
    } on ${post.source}`;
    listItem.appendChild(link);
    listItem.appendChild(pubDate);
    rssFeedList.appendChild(listItem);
  });
}

function addAndSortPosts(newPosts) {
  allPosts.push(...newPosts);
  allPosts.sort((a, b) => b.pubDate - a.pubDate);
  displayPosts(allPosts);
}

async function runFetch() {
  try {
    spinner.style.display = "inline-block";
    const opmlUrl = localStorage.getItem("opmlUrl");
    if (!opmlUrl) return;
    const opmlDoc = await fetchOPML(opmlUrl);
    const rssUrls = parseOPML(opmlDoc);

    await Promise.all(rssUrls.map(async (url) => {
      try {
        const rssDoc = await fetchFeed(url);
        const posts = parseFeed(rssDoc, url);
        addAndSortPosts(posts);
      } catch (e) {
        console.error(`Failed to fetch or parse RSS feed: ${url}`, e);
      }
    }));
  } catch (e) {
    console.error("Error:", e);
  } finally {
    spinner.style.display = "none";
  }
}

showButton.addEventListener("click", () => {
  opmlInput.value = localStorage.getItem("opmlUrl") || "";
  dialog.showModal();
});

closeButton.addEventListener("click", () => {
  dialog.close();
});

opmlForm.addEventListener("submit", (event) => {
  event.preventDefault();
  const opmlUrl = opmlInput.value.trim();
  if (opmlUrl) {
    localStorage.setItem("opmlUrl", opmlUrl);
    infoDialog.close();
    allPosts = [];
    rssFeedList.innerHTML = "";
    runFetch();
  }
});

runFetch();
