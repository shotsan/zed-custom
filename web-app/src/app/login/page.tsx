"use client";

import { useState } from "react";
import { motion } from "framer-motion";
import { ArrowRight, Hash } from "lucide-react";

export default function LoginPage() {
  const [email, setEmail] = useState("");
  const [otp, setOtp] = useState("");
  const [step, setStep] = useState(0); // 0 = email, 1 = otp
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  const handleEmailSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!email) return;
    setLoading(true);
    setError("");
    try {
      const res = await fetch("http://localhost:8080/auth/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email }),
      });
      if (res.ok) {
        setStep(1);
      } else {
        setError("Failed to send OTP. Is the backend running?");
      }
    } catch (err) {
      setError("Network error.");
    } finally {
      setLoading(false);
    }
  };

  const handleOtpSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!otp) return;
    setLoading(true);
    setError("");
    try {
      const res = await fetch("http://localhost:8080/auth/verify", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email, code: otp }),
      });
      if (res.ok) {
        const data = await res.json();
        // Redirect to Zed's native sign_in handler
        window.location.href = `zed-custom://sign_in?token=${data.token}`;
      } else {
        setError("Invalid OTP.");
      }
    } catch (err) {
      setError("Network error.");
    } finally {
      setLoading(false);
    }
  };

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
              Connect to Slack
            </h1>
            <p className="text-white mt-2 text-sm font-medium">
              Enter your credentials to sync channels
            </p>
          </div>

          <form className="w-full space-y-6" onSubmit={step === 0 ? handleEmailSubmit : handleOtpSubmit}>
            <div className="space-y-4">
              {error && <p className="text-red-500 text-sm font-bold text-center">{error}</p>}
              <div className="space-y-2">
                <label htmlFor="email" className="text-xs font-bold text-white uppercase tracking-widest">
                  Work Email
                </label>
                <input
                  type="email"
                  id="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  disabled={step === 1}
                  placeholder="name@company.com"
                  className="w-full rounded-lg border-2 border-white bg-black px-4 py-3 text-sm text-white placeholder:text-white/50 focus:border-white focus:outline-none focus:ring-2 focus:ring-white transition-colors disabled:opacity-50"
                />
              </div>

              {step === 1 && (
                <div className="space-y-2">
                  <label htmlFor="otp" className="text-xs font-bold text-white uppercase tracking-widest">
                    Verification Code
                  </label>
                  <input
                    type="text"
                    id="otp"
                    value={otp}
                    onChange={(e) => setOtp(e.target.value)}
                    placeholder="000000"
                    className="w-full rounded-lg border-2 border-white bg-black px-4 py-3 text-sm text-white placeholder:text-white/50 focus:border-white focus:outline-none focus:ring-2 focus:ring-white transition-colors"
                  />
                </div>
              )}
            </div>

            <button
              type="submit"
              disabled={loading}
              className="group flex w-full items-center justify-center gap-2 rounded-lg bg-white px-4 py-3 text-sm font-bold text-black transition-transform active:scale-[0.98] disabled:opacity-50"
            >
              <span>{loading ? "Please wait..." : step === 0 ? "Send Code" : "Authenticate"}</span>
              <ArrowRight className="h-4 w-4 transition-transform group-hover:translate-x-1" />
            </button>
          </form>
        </motion.div>
      </div>
    </main>
  );
}
