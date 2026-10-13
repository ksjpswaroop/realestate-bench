import React from "react";
import ReactDOM from "react-dom/client";
import { HashRouter, Routes, Route, Navigate } from "react-router-dom";
import App from "./App";
import DropPage from "./pages/DropPage";
import ExtractionsPage from "./pages/ExtractionsPage";
import SchemasPage from "./pages/SchemasPage";
import SettingsPage from "./pages/SettingsPage";
import "./styles/globals.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <HashRouter>
      <Routes>
        <Route path="/" element={<App />}>
          <Route index element={<DropPage />} />
          <Route path="extractions" element={<ExtractionsPage />} />
          <Route path="extractions/:id" element={<DropPage />} />
          <Route path="schemas" element={<SchemasPage />} />
          <Route path="settings" element={<SettingsPage />} />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Route>
      </Routes>
    </HashRouter>
  </React.StrictMode>
);
