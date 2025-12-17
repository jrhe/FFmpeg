FATE_DATA_URI-$(CONFIG_WAV_DEMUXER) += fate-data-uri-wav
fate-data-uri-wav: CMD = run ffprobe$(PROGSSUF)$(EXESUF) -v error -f wav \
  -show_entries stream=codec_name,codec_type,sample_rate,channels \
  -of compact=p=0:nk=1 \
  "data:audio/wav;base64,UklGRiQAAABXQVZFZm10IBAAAAABAAEAQB8AAIA+AAACABAAZGF0YQAAAAA="

FATE_FFPROBE += $(FATE_DATA_URI-yes)
fate-data-uri: $(FATE_DATA_URI-yes)

