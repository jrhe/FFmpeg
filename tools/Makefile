TOOLS = enc_recon_frame_test enum_options qt-faststart scale_slice_test trasher uncoded_frame
TOOLS-$(CONFIG_RUST_HLSWRITER) += bench_hlswriter
TOOLS-$(CONFIG_RUST_HLSPARSER) += bench_hlsparser
TOOLS-$(CONFIG_RUST_WEBVTT) += bench_webvtt
TOOLS-$(CONFIG_RUST_SUBRIP) += bench_subrip
TOOLS-$(CONFIG_LIBMYSOFA) += sofa2wavs
TOOLS-$(CONFIG_ZLIB) += cws2fws

tools/target_dec_%_fuzzer.o: tools/target_dec_fuzzer.c
	$(COMPILE_C) -DFFMPEG_DECODER=$*

tools/target_enc_%_fuzzer.o: tools/target_enc_fuzzer.c
	$(COMPILE_C) -DFFMPEG_ENCODER=$*

tools/target_bsf_%_fuzzer.o: tools/target_bsf_fuzzer.c
	$(COMPILE_C) -DFFMPEG_BSF=$*

tools/target_dem_%_fuzzer.o: tools/target_dem_fuzzer.c
	$(COMPILE_C) -DFFMPEG_DEMUXER=$* -DIO_FLAT=0

tools/target_dem_fuzzer.o: tools/target_dem_fuzzer.c
	$(COMPILE_C) -DIO_FLAT=1

tools/target_io_dem_fuzzer.o: tools/target_dem_fuzzer.c
	$(COMPILE_C) -DIO_FLAT=0

tools/target_sws_fuzzer.o: tools/target_sws_fuzzer.c
	$(COMPILE_C)

tools/target_swr_fuzzer.o: tools/target_swr_fuzzer.c
	$(COMPILE_C)

tools/target_hlsproto_fuzzer.o: tools/target_hlsproto_fuzzer.c
	$(COMPILE_C)

tools/target_hlsdemux_events_fuzzer.o: tools/target_hlsdemux_events_fuzzer.c
	$(COMPILE_C)

tools/target_webvtt_fuzzer.o: tools/target_webvtt_fuzzer.c
	$(COMPILE_C)

tools/target_subrip_fuzzer.o: tools/target_subrip_fuzzer.c
	$(COMPILE_C)

tools/target_microdvd_line_fuzzer.o: tools/target_microdvd_line_fuzzer.c
	$(COMPILE_C)

tools/target_ttml_extradata_fuzzer.o: tools/target_ttml_extradata_fuzzer.c
	$(COMPILE_C)

tools/target_mpl2_line_fuzzer.o: tools/target_mpl2_line_fuzzer.c
	$(COMPILE_C)

tools/target_vplayer_line_fuzzer.o: tools/target_vplayer_line_fuzzer.c
	$(COMPILE_C)

tools/target_jacosub_ts_fuzzer.o: tools/target_jacosub_ts_fuzzer.c
	$(COMPILE_C)

tools/target_subviewer_ts_fuzzer.o: tools/target_subviewer_ts_fuzzer.c
	$(COMPILE_C)

tools/target_scc_words_fuzzer.o: tools/target_scc_words_fuzzer.c
	$(COMPILE_C)

tools/target_stl_line_fuzzer.o: tools/target_stl_line_fuzzer.c
	$(COMPILE_C)

tools/bench_common.o: tools/bench_common.c
	$(COMPILE_C)

tools/bench_hlsparser.o: tools/bench_hlsparser.c
	$(COMPILE_C)

tools/bench_hlswriter.o: tools/bench_hlswriter.c
	$(COMPILE_C)

tools/bench_webvtt.o: tools/bench_webvtt.c
	$(COMPILE_C)

tools/bench_subrip.o: tools/bench_subrip.c
	$(COMPILE_C)

tools/enc_recon_frame_test$(EXESUF): tools/decode_simple.o
tools/venc_data_dump$(EXESUF): tools/decode_simple.o
tools/scale_slice_test$(EXESUF): tools/decode_simple.o
tools/decode_simple.o: | tools

tools/bench_hlsparser$(EXESUF): tools/bench_hlsparser.o tools/bench_common.o
	$(call LINK,$(LDFLAGS) $(LDEXEFLAGS) $(LD_O) $^ $(ELIBS))

tools/bench_hlswriter$(EXESUF): tools/bench_hlswriter.o tools/bench_common.o
	$(call LINK,$(LDFLAGS) $(LDEXEFLAGS) $(LD_O) $^ $(ELIBS))

tools/bench_webvtt$(EXESUF): tools/bench_webvtt.o tools/bench_common.o
	$(call LINK,$(LDFLAGS) $(LDEXEFLAGS) $(LD_O) $^ $(ELIBS))

tools/bench_subrip$(EXESUF): tools/bench_subrip.o tools/bench_common.o
	$(call LINK,$(LDFLAGS) $(LDEXEFLAGS) $(LD_O) $^ $(ELIBS))

OUTDIRS += tools

clean::
	$(RM) $(CLEANSUFFIXES:%=tools/%)

-include $(wildcard tools/*.d)
