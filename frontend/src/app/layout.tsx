import type { Metadata } from "next";
import localFont from "next/font/local";
import "./globals.css";

const manrope = localFont({ src: "./fonts/Manrope.ttf", variable: "--font-manrope", display: "swap", weight: "200 800" });

export const metadata: Metadata = {
  title: "ProcureOps AI — Autonomous Procurement & Treasury Cockpit",
  description:
    "Autonomous B2B purchasing agent governed by deterministic financial guardrails, integrated with PayPal REST v2 Sandbox APIs and AG Grid.",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className={manrope.variable}>
      <body
        className="antialiased"
      >
        {children}
      </body>
    </html>
  );
}
