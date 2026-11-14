export type TriageResult = {
  diagnosis: string;
  recommendedAction: string;
  priority: "LOW" | "MEDIUM" | "HIGH" | "CRITICAL";
  checks: string[];
  source: "transparent-ruleset-v1";
};

export function triageIssue(observation: string): TriageResult {
  const text = observation.toLowerCase();

  if (/no water|no flow|dry tap|pump.*not|not pumping|stopped pumping/.test(text)) {
    return {
      diagnosis: "Pump or delivery-path interruption",
      recommendedAction: "Isolate the power supply, inspect the pump controller and protection trips, verify water level, then have a qualified technician test the pump and delivery line.",
      priority: "HIGH",
      checks: ["Power isolation and controller fault state", "Water level and dry-run protection", "Pump current and delivery-line obstruction"],
      source: "transparent-ruleset-v1",
    };
  }

  if (/low pressure|weak flow|slow flow|reduced output/.test(text)) {
    return {
      diagnosis: "Reduced hydraulic output",
      recommendedAction: "Inspect filters, valves and pipe restrictions, compare measured flow with the operator's normal baseline, then check pump performance.",
      priority: "MEDIUM",
      checks: ["Filter and valve position", "Leakage or pipe restriction", "Measured flow versus operator baseline"],
      source: "transparent-ruleset-v1",
    };
  }

  if (/solar|inverter|controller|power|electrical|battery/.test(text)) {
    return {
      diagnosis: "Power-generation or control-system fault",
      recommendedAction: "Keep the system isolated until inspected. Have a qualified technician check the solar array, protection devices, inverter/controller alarms and battery connections.",
      priority: "HIGH",
      checks: ["Controller/inverter fault code", "DC isolator and protection devices", "Battery voltage and cable condition"],
      source: "transparent-ruleset-v1",
    };
  }

  if (/leak|burst|pipe|joint|water escaping/.test(text)) {
    return {
      diagnosis: "Water-line integrity issue",
      recommendedAction: "Isolate the affected line, locate the leak, inspect joints and fittings, and request a repair quote before restoring service.",
      priority: "HIGH",
      checks: ["Leak location and severity", "Valve isolation", "Pipe/joint condition and repair materials"],
      source: "transparent-ruleset-v1",
    };
  }

  if (/noise|vibration|bearing|heat|overheat/.test(text)) {
    return {
      diagnosis: "Possible mechanical wear or alignment fault",
      recommendedAction: "Stop the affected equipment if the noise or vibration is abnormal. Have a technician inspect bearings, mounting, alignment and temperature before restart.",
      priority: "MEDIUM",
      checks: ["Abnormal vibration/noise source", "Mounting and alignment", "Bearing and temperature condition"],
      source: "transparent-ruleset-v1",
    };
  }

  return {
    diagnosis: "Unclassified maintenance observation",
    recommendedAction: "Keep the asset under operator observation and request a qualified technician inspection. The coordinator cannot infer a sensor reading or confirm a fault from this description alone.",
    priority: "MEDIUM",
    checks: ["Record the exact symptom and when it began", "Capture a photo or short video if safe", "Ask a technician to inspect before authorizing spend"],
    source: "transparent-ruleset-v1",
  };
}
