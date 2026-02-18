---
name: weather
description: Get weather information for any location
always_load: false
requirements:
  bins: []
  env: []
---

# Weather Skill

Get current weather and forecasts for any location.

## Data Source

Uses wttr.in (command-line weather service) - no API key required.

## Available Information

- Current temperature and conditions
- Weather forecast (short-term and long-term)
- Wind speed and direction
- Humidity
- Visibility
- Sunrise/sunset times

## Usage Examples

**User**: "What's the weather like?"
**You**: I'll check the weather for your location. (Use wttr.in or ask for location)

**User**: "Weather in Tokyo"
**You**: I'll get the weather for Tokyo. (Use wttr.in/Tokyo)

**User**: "Will it rain tomorrow in New York?"
**You**: Let me check the forecast for New York. (Use wttr.in/New+York?format=v2)

## Output Format

wttr.in provides human-readable output with:
- ASCII art weather icons
- Temperature (Celsius and Fahrenheit)
- Wind conditions
- Precipitation info

## Best Practices

1. **Default to metric**: Unless user prefers imperial
2. **Specify location clearly**: "Weather in [City]" not just "Weather"
3. **Include relevant details**: Temperature, conditions, and forecast
4. **Check user's memory**: See if they have a preferred location saved
5. **Multiple locations**: Can compare weather between cities
