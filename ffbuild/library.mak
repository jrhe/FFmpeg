include $(SRC_PATH)/ffbuild/common.mak

# Optional Rust integration (already included by ffbuild/common.mak).

ifeq (,$(filter %clean config,$(MAKECMDGOALS)))
-include $(SUBDIR)lib$(NAME).version
endif

LIBVERSION := $(lib$(NAME)_VERSION)
LIBMAJOR   := $(lib$(NAME)_VERSION_MAJOR)
LIBMINOR   := $(lib$(NAME)_VERSION_MINOR)
INCINSTDIR := $(INCDIR)/lib$(NAME)

INSTHEADERS := $(INSTHEADERS) $(HEADERS:%=$(SUBDIR)%)

all-$(CONFIG_STATIC): $(SUBDIR)$(LIBNAME)  $(SUBDIR)lib$(FULLNAME).pc
all-$(CONFIG_SHARED): $(SUBDIR)$(SLIBNAME) $(SUBDIR)lib$(FULLNAME).pc

LIBOBJS := $(OBJS) $(SHLIBOBJS) $(STLIBOBJS) $(SUBDIR)%.h.o $(TESTOBJS)
$(LIBOBJS) $(LIBOBJS:.o=.s) $(LIBOBJS:.o=.i):   CPPFLAGS += -DHAVE_AV_CONFIG_H

ifdef CONFIG_SHARED
# In case both shared libs and static libs are enabled, it can happen
# that a user might want to link e.g. libavformat statically, but
# libavcodec and the other libs dynamically. In this case
# libavformat won't be able to access libavcodec's internal symbols,
# so that they have to be duplicated into the archive just like
# for purely shared builds.
# Test programs are always statically linked against their library
# to be able to access their library's internals, even with shared builds.
# Yet linking against dependent libraries still uses dynamic linking.
# This means that we are in the scenario described above.
# In case only static libs are used, the linker will only use
# one of these copies; this depends on the duplicated object files
# containing exactly the same symbols.
OBJS += $(SHLIBOBJS)
endif
$(SUBDIR)$(LIBNAME): $(OBJS) $(STLIBOBJS)
	$(RM) $@
ifeq ($(RESPONSE_FILES),yes)
	$(Q)echo $^ > $@.objs
	$(AR) $(ARFLAGS) $(AR_O) @$@.objs
else
	$(AR) $(ARFLAGS) $(AR_O) $^
endif
	$(RANLIB) $@
	-$(RM) $@.objs

install-headers: install-lib$(NAME)-headers install-lib$(NAME)-pkgconfig

install-libs-$(CONFIG_STATIC): install-lib$(NAME)-static
install-libs-$(CONFIG_SHARED): install-lib$(NAME)-shared

define RULES
$(TOOLS):     THISLIB = $(FULLNAME:%=$(LD_LIB))
$(TESTPROGS): THISLIB = $(SUBDIR)$(LIBNAME)

$(LIBOBJS): CPPFLAGS += -DBUILDING_$(NAME)

$(NAME)LINK_EXE_ARGS = $(LDFLAGS) $(LDEXEFLAGS)
$(NAME)LINK_SO_ARGS = $(SHFLAGS) $(LDFLAGS) $(LDSOFLAGS)
$(NAME)LINK_EXTRA = $(FFEXTRALIBS)

ifdef CONFIG_RUST_HLSWRITER
$(NAME)LINK_EXTRA += $(RUST_FFMPEG_HLSWRITER_LIB)
endif

ifdef CONFIG_RUST_HLSPARSER
$(NAME)LINK_EXTRA += $(RUST_FFMPEG_HLSPARSER_LIB)
endif

ifdef CONFIG_RUST_WEBVTT
$(NAME)LINK_EXTRA += $(RUST_FFMPEG_WEBVTT_LIB)
endif

ifdef CONFIG_RUST_SUBRIP
$(NAME)LINK_EXTRA += $(RUST_FFMPEG_SUBRIP_LIB)
endif

ifdef CONFIG_RUST_HLSDEMUX_PARSER
$(NAME)LINK_EXTRA += $(RUST_FFMPEG_HLSPARSER_LIB)
endif

ifdef CONFIG_RUST_MICRODVD
$(NAME)LINK_EXTRA += $(RUST_FFMPEG_MICRODVD_LIB)
endif

ifdef CONFIG_RUST_TTML
$(NAME)LINK_EXTRA += $(RUST_FFMPEG_TTML_LIB)
endif

ifdef CONFIG_RUST_MPL2
$(NAME)LINK_EXTRA += $(RUST_FFMPEG_MPL2_LIB)
endif

ifdef CONFIG_RUST_VPLAYER
$(NAME)LINK_EXTRA += $(RUST_FFMPEG_VPLAYER_LIB)
endif

ifdef CONFIG_RUST_JACOSUB
$(NAME)LINK_EXTRA += $(RUST_FFMPEG_JACOSUB_LIB)
endif

ifdef CONFIG_RUST_SUBVIEWER
$(NAME)LINK_EXTRA += $(RUST_FFMPEG_SUBVIEWER_LIB)
endif

ifdef CONFIG_RUST_SUBVIEWER1
$(NAME)LINK_EXTRA += $(RUST_FFMPEG_SUBVIEWER_LIB)
endif

ifdef CONFIG_RUST_SCC
$(NAME)LINK_EXTRA += $(RUST_FFMPEG_SCC_LIB)
endif

ifdef CONFIG_RUST_STL
$(NAME)LINK_EXTRA += $(RUST_FFMPEG_STL_LIB)
endif

ifdef CONFIG_RUST_LRC
$(NAME)LINK_EXTRA += $(RUST_FFMPEG_LRC_LIB)
endif


$(TESTPROGS) $(TOOLS): %$(EXESUF): %.o
	$$(call LINK,$$(call $(NAME)LINK_EXE_ARGS) $$(LD_O) $$(filter %.o,$$^) $$(THISLIB) $$(call $(NAME)LINK_EXTRA) $$(EXTRALIBS-$$(*F)) $$(ELIBS))

$(SUBDIR)lib$(NAME).version: $(SUBDIR)version.h $(SUBDIR)version_major.h | $(SUBDIR)
	$$(M) $$(SRC_PATH)/ffbuild/libversion.sh $(NAME) $$^ > $$@

$(SUBDIR)lib$(FULLNAME).pc: $(SUBDIR)version.h ffbuild/config.sh | $(SUBDIR)
	$$(M) $$(SRC_PATH)/ffbuild/pkgconfig_generate.sh $(NAME) "$(DESC)"

$(SUBDIR)lib$(NAME).ver: $(SUBDIR)lib$(NAME).v $(OBJS)
	$$(M)sed 's/MAJOR/$(lib$(NAME)_VERSION_MAJOR)/' $$< | $(VERSION_SCRIPT_POSTPROCESS_CMD) > $$@

$(SUBDIR)$(SLIBNAME): $(SUBDIR)$(SLIBNAME_WITH_MAJOR)
	$(Q)cd ./$(SUBDIR) && $(LN_S) $(SLIBNAME_WITH_MAJOR) $(SLIBNAME)

$(SUBDIR)$(SLIBNAME_WITH_MAJOR): $(OBJS) $(SHLIBOBJS) $(SUBDIR)lib$(NAME).ver
	$(SLIB_CREATE_DEF_CMD)
ifeq ($(RESPONSE_FILES),yes)
	$(Q)echo $$(filter %.o,$$^) > $$@.objs
	$$(call LINK,$$(call $(NAME)LINK_SO_ARGS) $$(LD_O) @$$@.objs $$(call $(NAME)LINK_EXTRA))
else
	$$(call LINK,$$(call $(NAME)LINK_SO_ARGS) $$(LD_O) $$(filter %.o,$$^) $$(call $(NAME)LINK_EXTRA))
endif
	$(SLIB_EXTRA_CMD)
	-$(RM) $$@.objs

ifdef SUBDIR
$(SUBDIR)$(SLIBNAME_WITH_MAJOR): $(DEP_LIBS)
endif

clean::
	$(RM) $(addprefix $(SUBDIR),$(CLEANFILES) $(CLEANSUFFIXES) $(LIBSUFFIXES)) \
	    $(CLEANSUFFIXES:%=$(SUBDIR)$(ARCH)/%) $(CLEANSUFFIXES:%=$(SUBDIR)tests/%)

install-lib$(NAME)-shared: $(SUBDIR)$(SLIBNAME)
	$(Q)mkdir -p "$(SHLIBDIR)"
	$$(INSTALL) -m 755 $$< "$(SHLIBDIR)/$(SLIB_INSTALL_NAME)"
	$$(STRIP) "$(SHLIBDIR)/$(SLIB_INSTALL_NAME)"
	$(Q)$(foreach F,$(SLIB_INSTALL_LINKS),(cd "$(SHLIBDIR)" && $(LN_S) $(SLIB_INSTALL_NAME) $(F));)
	$(if $(SLIB_INSTALL_EXTRA_SHLIB),$$(INSTALL) -m 644 $(SLIB_INSTALL_EXTRA_SHLIB:%=$(SUBDIR)%) "$(SHLIBDIR)")
	$(if $(SLIB_INSTALL_EXTRA_LIB),$(Q)mkdir -p "$(LIBDIR)")
	$(if $(SLIB_INSTALL_EXTRA_LIB),$$(INSTALL) -m 644 $(SLIB_INSTALL_EXTRA_LIB:%=$(SUBDIR)%) "$(LIBDIR)")

install-lib$(NAME)-static: $(SUBDIR)$(LIBNAME)
	$(Q)mkdir -p "$(LIBDIR)"
	$$(INSTALL) -m 644 $$< "$(LIBDIR)"
	$(LIB_INSTALL_EXTRA_CMD)

install-lib$(NAME)-headers: $(addprefix $(SUBDIR),$(HEADERS) $(BUILT_HEADERS))
	$(Q)mkdir -p "$(INCINSTDIR)"
	$$(INSTALL) -m 644 $$^ "$(INCINSTDIR)"

install-lib$(NAME)-pkgconfig: $(SUBDIR)lib$(FULLNAME).pc
	$(Q)mkdir -p "$(PKGCONFIGDIR)"
	$$(INSTALL) -m 644 $$^ "$(PKGCONFIGDIR)"

uninstall-libs::
	-$(RM) "$(SHLIBDIR)/$(SLIBNAME_WITH_MAJOR)" \
	       "$(SHLIBDIR)/$(SLIBNAME)"            \
	       "$(SHLIBDIR)/$(SLIBNAME_WITH_VERSION)"
	-$(RM)  $(SLIB_INSTALL_EXTRA_SHLIB:%="$(SHLIBDIR)/%")
	-$(RM)  $(SLIB_INSTALL_EXTRA_LIB:%="$(LIBDIR)/%")
	-$(RM) "$(LIBDIR)/$(LIBNAME)"

uninstall-headers::
	$(RM) $(addprefix "$(INCINSTDIR)/",$(HEADERS) $(BUILT_HEADERS))
	-rmdir "$(INCINSTDIR)"

uninstall-pkgconfig::
	$(RM) "$(PKGCONFIGDIR)/lib$(FULLNAME).pc"
endef

$(eval $(RULES))

$(TOOLS):     $(DEP_LIBS) $(SUBDIR)$($(CONFIG_SHARED:yes=S)LIBNAME)
$(TESTPROGS): $(DEP_LIBS) $(SUBDIR)$(LIBNAME)

ifdef HAVE_FFMPEG_RUST
ifeq ($(strip $(CONFIG_RUST_HLSWRITER)$(CONFIG_RUST_HLSPARSER)$(CONFIG_RUST_HLSDEMUX_PARSER)$(CONFIG_RUST_WEBVTT)$(CONFIG_RUST_SUBRIP)$(CONFIG_RUST_MICRODVD)$(CONFIG_RUST_TTML)$(CONFIG_RUST_MPL2)$(CONFIG_RUST_VPLAYER)$(CONFIG_RUST_JACOSUB)$(CONFIG_RUST_SUBVIEWER)$(CONFIG_RUST_SUBVIEWER1)$(CONFIG_RUST_SCC)$(CONFIG_RUST_STL)$(CONFIG_RUST_LRC)),)
else
$(TESTPROGS) $(TOOLS): | rust-libs
endif
endif

testprogs: $(TESTPROGS)
