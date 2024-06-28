const DEFAULT_CORS_PROXY = "https://api.allorigins.win/raw?url=";

const dialog = document.getElementById("info-dialog");
const showButton = document.getElementById("show-button");
const closeButton = document.getElementById("close-button");

const form = document.getElementById("form");
const opmlInput = document.getElementById("opml-url");

const corsInput = document.getElementById("cors-url");
let corsProxy = localStorage.getItem("corsProxy") || DEFAULT_CORS_PROXY;
corsInput.value = corsProxy;

const rssFeedList = document.getElementById("fd");
let allPosts = [];

function timeSince(date) {
  const seconds = Math.floor((new Date() - date) / 1000),
    intervals = [
      { label: " years ago", seconds: 365 * 24 * 60 * 60 },
      { label: " months ago", seconds: 30 * 24 * 60 * 60 },
      { label: " days ago", seconds: 24 * 60 * 60 },
      { label: " hours ago", seconds: 60 * 60 },
      { label: " minutes ago", seconds: 60 },
    ];
  for (let i = 0; i < intervals.length; i++) {
    const interval = intervals[i];
    const result = Math.floor(seconds / interval.seconds);
    if (result >= 1) return result + interval.label;
  }
  return seconds + " seconds ago";
}

const getDomain = (url) => new URL(url).hostname;

async function fetchWithCORSRetry(url) {
  try {
    return await fetch(url);
  } catch (_error) {
    return await fetch(corsProxy + url);
  }
}

async function fetchDomParser(url, contentType) {
  const res = await fetchWithCORSRetry(url);
  if (!res.ok) throw new Error("Failed to fetch");
  return new DOMParser().parseFromString(await res.text(), contentType);
}

const parseOPML = (doc) =>
  Array.from(doc.querySelectorAll('outline[type="rss"]')).map((o) =>
    o.getAttribute("xmlUrl")
  );

function parseFeed(doc, source) {
  return doc.documentElement.nodeName === "feed"
    ? parseAtom(doc, source)
    : parseRSS(doc, source);
}

const parseRSS = (doc, source) =>
  Array.from(doc.querySelectorAll("item")).map((item) => ({
    title: item.querySelector("title").textContent,
    link: item.querySelector("link").textContent,
    pubDate: new Date(item.querySelector("pubDate").textContent),
    source: getDomain(source),
  }));

const parseAtom = (doc, source) =>
  Array.from(doc.querySelectorAll("entry")).map((entry) => ({
    title: entry.querySelector("title").textContent,
    link: entry.querySelector("link").getAttribute("href"),
    pubDate: new Date(
      entry.querySelector("updated").textContent ||
        entry.querySelector("published").textContent,
    ),
    source: getDomain(source),
  }));

function display(posts) {
  rssFeedList.innerHTML = "";
  posts.forEach((post) => {
    const li = document.createElement("li");
    const a = document.createElement("a");
    const span = document.createElement("span");
    a.href = post.link;
    a.textContent = post.title;
    span.className = "entry-sub";
    span.innerHTML = `<br>published ${
      timeSince(post.pubDate)
    } on ${post.source}`;
    li.appendChild(a);
    li.appendChild(span);
    rssFeedList.appendChild(li);
  });
}

function addAndSortPosts(newPosts) {
  allPosts.push(...newPosts);
  allPosts.sort((a, b) => b.pubDate - a.pubDate);
  display(allPosts);
}

async function runFetch() {
  try {
    spinner.style.display = "inline-block";
    const url = localStorage.getItem("opmlUrl");
    if (!url) return;
    const doc = await fetchDomParser(url, "text/xml"),
      rssUrls = parseOPML(doc);
    await Promise.all(rssUrls.map(async (url) => {
      try {
        const rssDoc = await fetchDomParser(url, "application/xml");
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

closeButton.addEventListener("click", () => dialog.close());

form.addEventListener("submit", (e) => {
  e.preventDefault();
  const url = opmlInput.value.trim();
  corsProxy = corsInput.value.trim();
  if (url) {
    localStorage.setItem("corsProxy", corsProxy);
    localStorage.setItem("opmlUrl", url);
    dialog.close();
    allPosts = [];
    rssFeedList.innerHTML = "";
    runFetch();
  }
});

runFetch();
