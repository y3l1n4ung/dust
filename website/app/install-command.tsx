"use client";

import { useEffect, useState } from "react";

const command = "curl -fsSL https://raw.githubusercontent.com/y3l1n4ung/dust/main/install.sh | bash";

export function InstallCommand() {
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (!copied) return;
    const timer = window.setTimeout(() => setCopied(false), 1800);
    return () => window.clearTimeout(timer);
  }, [copied]);

  async function copyCommand() {
    await navigator.clipboard.writeText(command);
    setCopied(true);
  }

  return (
    <div className="install-command">
      <code><span>$</span> {command}</code>
      <button type="button" onClick={copyCommand} aria-live="polite">
        {copied ? "Copied" : "Copy"}
      </button>
    </div>
  );
}
