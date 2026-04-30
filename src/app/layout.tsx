import type { Metadata } from "next";
import "./globals.css";
import "@fontsource/iosevka/400.css";
import "@fontsource/iosevka/500.css";
import "@fontsource/iosevka/600.css";
import { Providers } from "@/components/providers";
import { Toaster } from "@/components/ui/sonner";
import { TauriEventListener } from "@/components/tauri-event-listener";

export const metadata: Metadata = {
  title: "Takoyaki",
  description: "Octatrack backup and project manager",
};

export default function RootLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en">
      <head>
        <meta name="color-scheme" content="dark" />
      </head>
      <body className="min-h-screen">
        <Providers>
          <div id="main-content" className="h-screen overflow-hidden">
            {children}
          </div>
          <TauriEventListener />
          <Toaster position="bottom-right" />
        </Providers>
      </body>
    </html>
  );
}
