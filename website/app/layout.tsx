import type { Metadata } from "next";
import { headers } from "next/headers";
import "./globals.css";

export async function generateMetadata(): Promise<Metadata> {
  const requestHeaders = await headers();
  const host = requestHeaders.get("x-forwarded-host") ?? requestHeaders.get("host") ?? "localhost:3000";
  const protocol = requestHeaders.get("x-forwarded-proto") ?? (host.startsWith("localhost") ? "http" : "https");
  const origin = protocol + "://" + host;
  const image = origin + "/og.png";

  return {
    title: "Dust — Built to make developers and AI agents happy",
    description: "Dust is Rust-powered code generation for Dart and Flutter, built for human developers and AI coding agents. It handles repetitive and complex code so you can focus on your product.",
    icons: {
      icon: "/dust-mascot.png",
      apple: "/dust-mascot.png",
    },
    openGraph: {
      title: "Dust — Built to make developers and AI agents happy",
      description: "Dust is Rust-powered code generation for Dart and Flutter, built for human developers and AI coding agents. It handles repetitive and complex code so you can focus on your product.",
      type: "website",
      url: origin,
      images: [{ url: image, width: 1728, height: 910, alt: "Dust — Built to make developers and AI agents happy" }],
    },
    twitter: {
      card: "summary_large_image",
      title: "Dust — Built to make developers and AI agents happy",
      description: "Dust is Rust-powered code generation for Dart and Flutter, built for human developers and AI coding agents. It handles repetitive and complex code so you can focus on your product.",
      images: [image],
    },
  };
}

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return <html lang="en"><body>{children}</body></html>;
}
