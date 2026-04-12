/**
 * Build a browser URL for a given commit on common Git hosts.
 * Returns null if remote or sha is unusable.
 */
export function commitBrowseUrl(remote: string | undefined, commitSha: string | undefined): string | null {
  const r = remote?.trim() ?? "";
  const sha = commitSha?.trim().split(/\s+/)[0] ?? "";
  if (!r || !sha || sha === "—") return null;

  let host: string;
  let pathNoLeading: string;

  const ssh = /^git@([^:]+):(.+)$/.exec(r);
  if (ssh) {
    host = ssh[1].toLowerCase();
    pathNoLeading = ssh[2].replace(/\.git$/i, "").replace(/\\/g, "/");
  } else if (r.startsWith("ssh://")) {
    try {
      const u = new URL(r);
      host = (u.hostname || "").toLowerCase();
      pathNoLeading = (u.pathname || "").replace(/^\/+/, "").replace(/\.git$/i, "");
    } catch {
      return null;
    }
  } else {
    try {
      const u = new URL(r);
      if (!u.protocol.startsWith("http")) return null;
      host = u.hostname.toLowerCase();
      pathNoLeading = u.pathname.replace(/^\/+/, "").replace(/\.git$/i, "");
    } catch {
      return null;
    }
  }

  if (!host || !pathNoLeading) return null;

  const base = `https://${host}/${pathNoLeading}`;

  if (host.includes("gitlab")) {
    return `${base}/-/commit/${sha}`;
  }
  if (host.includes("bitbucket.org")) {
    return `${base}/commits/${sha}`;
  }
  // GitHub, Gitee, Codeberg, and unknown — same path pattern
  return `${base}/commit/${sha}`;
}
