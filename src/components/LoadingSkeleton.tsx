"use client";

import React from "react";

export function Skeleton({ className = "" }) {
  return (
    <div className={`bg-gray-700 animate-pulse rounded ${className}`} />
  );
}
