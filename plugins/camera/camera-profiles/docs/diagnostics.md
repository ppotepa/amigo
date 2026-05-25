# Camera Profiles Diagnostics

Primary channel: `camera-profiles.catalog`.

## Profile Format
`format_camera_profile_2d` reports:
- profile id and label
- lens profile id or `none`
- film profile id or `none`
- focus distance in meters or `none`

## Checks
- Missing profile ids should be reported by the profile consumer.
- Catalog diagnostics should describe selected data, not patch over missing
  camera configuration.
- Quality profile diagnostics should preserve the chosen preset name and derived
  buffer quality values.
