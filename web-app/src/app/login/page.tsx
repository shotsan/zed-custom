"use client";

import { motion } from "framer-motion";
import { ArrowRight, Hash } from "lucide-react";

export default function LoginPage() {
  return (
    <main className="min-h-screen flex items-center justify-center bg-black text-white font-sans selection:bg-white/30">
      <div className="w-full max-w-sm px-6">
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, ease: "easeOut" }}
          className="flex flex-col items-center"
        >
          <div className="mb-8 flex h-12 w-12 items-center justify-center rounded-xl bg-black border-2 border-white">
            <Hash className="h-6 w-6 text-white" />
          </div>

          <div className="text-center mb-10 w-full">
            <h1 className="text-2xl font-bold tracking-tight text-white">
              Connect to Zed Custom
            </h1>
            <p className="text-white mt-2 text-sm font-medium">
              Sign in to your enterprise workspace
            </p>
          </div>

          <div className="w-full space-y-6">
            <a
              href="https://api.zedcustom.dev/auth/login"
              className="group flex w-full items-center justify-center gap-2 rounded-lg bg-white px-4 py-3 text-sm font-bold text-black transition-transform active:scale-[0.98] hover:bg-gray-200"
            >
              <span>Sign In</span>
              <ArrowRight className="h-4 w-4 transition-transform group-hover:translate-x-1" />
            </a>
          </div>
        </motion.div>
      </div>
    </main>
  );
}
