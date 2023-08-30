"use client";
import styles from "./page.module.css";
import React, { useState, useEffect } from "react";

export default function Home() {
  const calculatePercentage = (used, total) => (used / total) * 100;
  const [data, setData] = useState({ cpu: {}, disk: {}, ram: {}, swap: {} });
  useEffect(() => {
    const ws = new WebSocket("ws://127.0.0.1:9000/ws");
    ws.onmessage = function (event) {
      const json = JSON.parse(event.data);
      try {
        setData(json);
      } catch (err) {
        console.log(err);
      }
    };
    return () => {
      ws.close();
    };
  }, []);

  return (
    <div className={styles.App}>
      <h1 className={styles.h1}>System Monitor</h1>
      <h2 className={styles.h2}>CPU Usage</h2>
      <ul className={styles.ul}>
        {Object.entries(data.cpu).map(([core, usage]) => (
          <li className={styles.li} key={core}>
            Core {core}:
            <div className={styles.bar}>
              <div className={styles.fill} style={{ width: `${usage}%` }}></div>
            </div>
            {usage.toFixed(2)}%
          </li>
        ))}
      </ul>
      <h2 className={styles.h2}>Disk Usage</h2>
      <div className={styles.bar}>
        <div
          className={styles.fill}
          style={{
            width: `${calculatePercentage(data.disk.used, data.disk.total)}%`,
          }}
        ></div>
      </div>
      Used: {bytesToGB(data.disk.used)}, Total: {bytesToGB(data.disk.total)}
      <h2 className={styles.h2}>RAM Usage</h2>
      <div className={styles.bar}>
        <div
          className={styles.fill}
          style={{
            width: `${calculatePercentage(data.ram.used, data.ram.total)}%`,
          }}
        ></div>
      </div>
      Used: {bytesToGB(data.ram.used)}, Total: {bytesToGB(data.ram.total)}
      <h2 className={styles.h2}>Swap Usage</h2>
      <div className={styles.bar}>
        <div
          className={styles.fill}
          style={{
            width: `${calculatePercentage(data.swap.used, data.swap.total)}%`,
          }}
        ></div>
      </div>
      Used: {bytesToGB(data.swap.used)}, Total: {bytesToGB(data.swap.total)}
    </div>
  );
}

// function transform bytes to gb with 2 decimals
function bytesToGB(bytes) {
  return (bytes / 1024 / 1024 / 1024).toFixed(2);
}


