#!/usr/bin/Rscript

suppressPackageStartupMessages(library('tidyverse', warn.conflicts = F))
library('magrittr', warn.conflicts = F)
library('withr', warn.conflicts = F)
library('glue', warn.conflicts = F)
library('crayon', warn.conflicts = F)
library('optparse', warn.conflicts = F)
suppressPackageStartupMessages(library('viridis', warn.conflicts = F))


options(width=120)
printf = function(...) cat(sprintf(...))
info <- function(...) printf(blue(...))
winfo <- function(...) printf(yellow(...))

ns_to_ms <- function(x) {
  x / 1e6
}
gmean <- function(x) {
  exp(mean(log(x)))
}
hmean <- function(x) {
  1 / mean(1 / x)
}

# binomial proportion confidence interval
bprop.ci.err <- function(x, n_samples, level = 0.95) {
  q <- qnorm(1 - ((1 - level) / 2))
  q * sqrt((x * (1 - x)) / n_samples)
}

# =============================================================================
# Fault Models:
# 0 = BitFlip
# 1 = BufferOverflow(API)
# 2 = BufferOverflowMalloc(Data)
# 3 = DataCorruption(Data)
# 4 = InvalidPointer(Res)
# 5 = RaceCondition(Timing)
# =============================================================================
root_input_dir <- '.'

input_dirs <- c('kmeans', 'pca', 'blackscholes', 'quicksort', 'swaptions')
result_type <- 'tsan-p-fi-results'
keep_faultmodels <- c(0, 1, 2, 3, 4)
fm_names <-
  c(
    '0' = 'BitFlip',
    '1' = 'FileSize',
    '2' = 'MallSize',
    '3' = 'CallCorr',
    '4' = 'InvalPtr',
    '5' = 'RaceCond'
  )
cut_n_faultmodel <- 5000
# =============================================================================


cmd_options <- list()
parser <- OptionParser(usage = "%prog DATA_ROOT_DIR", option_list = cmd_options)
args <- parse_args(parser, positional_arguments = c(0, 1))
if(length(args$args) > 0) {
  root_input_dir <- args$args[[1]]
}

check_read_issues <- function(x) {
  chk <- function(y) {
    prob <- problems(y)
    if (nrow(prob)) {
      append(capture.output(print(prob))[c(-1,-3)], '-----')
    }
  }
  iss <- x %>%
    group_by(indir) %>%
    do(lol=chk(.$data[[1]])) %>%
    ungroup() %>%
    summarize(all=paste(unlist(lol), collapse='\n'))
  iss <- unlist(iss$all)
  if (iss != '') {
    winfo('There were parsing issues:\n==============================================\n')
    winfo(iss)
    winfo('\n==============================================\n')
  }
  x
}


message(glue('Reading data from \'{root_input_dir}/{{{str_c(input_dirs, collapse = \', \')}}\'...'))
csv_data <- tibble(indir=input_dirs) %>%
  mutate(data=map(indir, ~ read_csv(glue('{root_input_dir}/{.}/{.}-{result_type}.csv'), col_names=T, col_types='ciiclclcc'))) %>%
  check_read_issues() %>%
  select(data) %>%
  unnest(data) %>%
  rename(target = Benchmark, faultmodel = `Fault-model`, runid = `#`, deviation = Deviation,
         odeviation = `Output-Deviation`, exit = `Exit-code`, factive = `Fault-active`,
         time_sym = Symbolification, time_cmp = Comparison) %>%
  mutate(target = factor(target), faultmodel = factor(faultmodel), deviation = factor(deviation)) %>%
  mutate(time_sym = ns_to_ms(as.double(time_sym)), time_cmp = ns_to_ms(as.double(time_cmp)))

if(nrow(problems(csv_data))) {
  warning(sprintf('%d Problems with data import detected.', nrow(problems(csv_data))))
  info('List of data import issues:\n----\n')
  print(problems(csv_data))
  info('----\n')
}


# # add fake data as long as we are still waiting for results
# warning('FAKE DATA still in place; you can ignore most other warnings!')
# csv_data <-
#   csv_data %>% complete(
#     target = c('blackscholes', 'pca', 'kmeans', 'fake-swaptions', 'quicksort'),
#     faultmodel,
#     fill = list(
#       factive = T,
#       deviation = 'data-dev',
#       odeviation = F,
#       exit = 1
#     )
#   )

# keep only the wished for fault models and name them
csv_data <- csv_data %>%
  filter(faultmodel %in% keep_faultmodels) %>%
  mutate(faultmodel = recode_factor(faultmodel, !!!fm_names, .ordered = T))

message('Basic sanity checks...')
dat.count.uniq <- csv_data %>%
  select(target, faultmodel, deviation, exit) %>%
  summarize_all(n_distinct)
info('Number of unique values in data:\n----\n')
print(dat.count.uniq, n=Inf)
info('----\n')
num_targets <- dat.count.uniq$target
num_faultmodels <- dat.count.uniq$faultmodel
num_deviations <- dat.count.uniq$deviation

check_runs_per_target <- function() {
  dat.count.runs <- csv_data %>%
    count(target)
  info('Number of runs per target:\n----\n')
  print(dat.count.runs)
  info('----\n')
  if (length(unique(dat.count.runs$n)) != 1) {
    warning('Detected different number of runs across targets!')
  }
}
check_runs_per_target()

check_runs_per_faultmodel <- function() {
  dat.count.runsfm <- csv_data %>%
    count(target, faultmodel)
  if (length(unique(dat.count.runsfm$n)) != 1) {
    warning('Detected different number of runs across targets and/or fault models!')
  } else {
    info('Number of runs per fault model:\n----\n')
    info(glue('{unique(dat.count.runsfm$n)}'), '\n')
    info('----\n')
  }
}
check_runs_per_faultmodel()

check_dead_runs <- function() {
  dat.count.runsact <- csv_data %>%
    count(factive)
  if (nrow(dat.count.runsact) != 1) {
    dat.noact <- csv_data %>%
      select(target, faultmodel, runid, deviation, exit, factive) %>%
      filter(factive == F)
    warning(yellow(sprintf(
      'Detected %d runs w/o fault activation!', nrow(dat.noact)
    )))
    
    dat.noact.target <- dat.noact %>% count(target)
    dat.noact.dev <- dat.noact %>% count(deviation)
    info('Number of dead runs per target:\n----\n')
    print(dat.noact.target)
    info('----\n')
    info('Number of dead runs per deviation:\n----\n')
    print(dat.noact.dev)
    info('----\n')
  }
}
check_dead_runs()

message('Pre-processing...')
csv_data %<>% filter(factive == T)
min_runs_per_fm <- csv_data %>%
  group_by(target, faultmodel) %>%
  count() %>%
  ungroup() %>%
  summarize(m = min(n))
min_runs_per_fm <- min_runs_per_fm$m
# fix nmuber of runs per fault model
csv_data <- csv_data %>%
  group_by(target, faultmodel) %>%
  arrange(target, faultmodel) %>%
  slice(1:cut_n_faultmodel) %>%
  ungroup()

check_runs_per_target()
check_runs_per_faultmodel()
check_dead_runs()

theme_set(
  theme_minimal(base_size = 12) + theme(
    plot.margin = margin(5, 5, 5, 5),
    panel.grid.major.x = element_blank(),
    legend.position = 'right',
    legend.title = element_blank(),
    legend.margin = margin(0, 0, 0, 0),
    legend.box.spacing = unit(0, 'pt'),
    legend.text = element_text(margin = margin(l = 2, unit = 'pt')),
    legend.key.size = unit(5, 'mm'),
    axis.text.x = element_text(size = 10),
    plot.title = element_text(hjust = 0.5)
  )
)
scale_colour_discrete <- function(...) scale_color_viridis(discrete = T, ...)
scale_fill_discrete <- function(...) scale_fill_viridis(discrete = T, ...)

# csv_data %>%
#   group_by(target) %>%
#   arrange(target, faultmodel, runid) %>%
#   ggplot(aes(x = runid, y = faultmodel)) + ggtitle('Experiment Results Time Series per Target and FM') +
#   geom_raster(aes(fill = deviation)) + facet_grid(target ~ .)

dat.counts.all <- csv_data %>%
  group_by(target, faultmodel, deviation) %>%
  summarize(devn = n()) %>%
  complete(nesting(target, faultmodel), deviation, fill=list(devn=0))
dat.counts.merged <- csv_data %>%
  mutate(deviation = fct_recode(deviation, 'data-dev' = 'addr-dev')) %>%
  mutate(deviation = fct_recode(deviation, 'data-dev' = 'control-dev')) %>%
  group_by(target, faultmodel, deviation) %>%
  summarize(devn = n()) %>%
  complete(nesting(target, faultmodel), deviation, fill=list(devn=0))

# dat.counts.all %>%
#   ggplot(aes(faultmodel, devn, fill = deviation)) +
#   geom_col(position = position_dodge(width = 0.9)) +
#   geom_text(
#     aes(faultmodel, devn, label = sprintf('%d', devn)),
#     vjust = 0.5,
#     hjust = 0,
#     angle = 90,
#     position = position_dodge(width = 0.9)
#   ) +
#   ggtitle('Absolute Deviations per FM and Target (All Classes)') +
#   facet_grid(target ~ .)

# dat.counts.merged %>%
#   ggplot(aes(faultmodel, devn, fill = deviation)) +
#   geom_col(position = position_dodge(width = 0.9)) +
#   geom_text(
#     aes(faultmodel, devn, label = sprintf('%d', devn)),
#     vjust = 0.5,
#     hjust = 0,
#     angle = 90,
#     position = position_dodge(width = 0.9)
#   ) +
#   ggtitle('Absolute Deviations per FM and Target (Yes/No)') +
#   facet_grid(target ~ .)

dat.trans.raw <- csv_data %>%
  mutate(cleanexit = exit == 0) %>%
  mutate(hasdev = deviation != 'no-dev', hasodev = odeviation) %>%
  mutate(issdc = (hasodev & cleanexit)) %>%
  select(-runid, -factive, -time_sym, -time_cmp, -deviation, -odeviation, -exit) %>%
  group_by(target) %>%
  mutate(n_in_target = n()) %>%
  group_by(target, faultmodel) %>%
  mutate(n_in_fm = n())
dat.trans.classes <- dat.trans.raw %>%
  mutate(fclass = replace(cleanexit, cleanexit, 'Benign'),
         fclass = replace(fclass, issdc, 'SDC'),
         fclass = replace(fclass, !cleanexit, 'Crash')) %>%
  mutate(fclass = factor(fclass)) %>%
  select(-cleanexit, -hasodev, -issdc)

dat.trans.classes.counts <- dat.trans.classes %>%
  group_by(target, faultmodel, hasdev, fclass) %>%
  summarize(n = n(), n_in_target = max(n_in_target), n_in_fm = max(n_in_fm))

dat.trans.classes.counts.check <- dat.trans.classes.counts %>%
  ungroup() %>%
  filter(!hasdev) %>%
  select(target, fclass) %>%
  group_by(target, fclass) %>%
  distinct()
if(nrow(dat.trans.classes.counts.check) != num_targets) {
  warning('Detected cases w/o TSAN deviation that are NOT benign!!')
  print(dat.trans.classes.counts.check, n=Inf)
}

dat.trans.classes.counts.rel <- dat.trans.classes.counts %>%
  mutate(n = n / n_in_fm) %>%
  select(-n_in_target) %>%
  rename(nn = n_in_fm)
# comment in to get mean over all fault models
# tmp1 <- dat.trans.classes.counts.rel %>%
#   ungroup() %>%
#   bind_rows(
#     .,
#     group_by(., target, hasdev, fclass) %>%
#       summarize(
#         faultmodel = 'mean',
#         n = mean(n),
#         nn = mean(nn)
#       )
#   ) %>%
#   arrange(target) %>%
#   mutate(faultmodel = factor(faultmodel), faultmodel = fct_inorder(faultmodel)) %>%
#   group_by(target, faultmodel, hasdev)
# dat.trans.classes.counts.rel <- tmp1

dat.trans.classes.counts.rel.ci <- dat.trans.classes.counts.rel %>%
  summarize(s = sum(n), nn = max(nn)) %>%
  mutate(ci = bprop.ci.err(s, nn)) %>%
  mutate(ymin = s - ci, ymax = s + ci) %>%
  filter(hasdev)
  
dat.trans.classes.counts.rel.target <- dat.trans.classes.counts %>%
  group_by(target, hasdev, fclass) %>%
  mutate(n = n / n_in_target) %>%
  summarize(n = sum(n), nn = max(n_in_target))
dat.trans.classes.counts.rel.target.ci <- dat.trans.classes.counts.rel.target %>%
  summarize(s = sum(n), nn=max(nn)) %>%
  mutate(ci = bprop.ci.err(s, nn)) %>%
  mutate(ymin = s - ci, ymax = s + ci) %>%
  filter(hasdev)

paper_theme <- theme_minimal(base_size = 8) + theme(
  plot.margin = margin(1, 1, 1, 1),
  panel.grid.major.x = element_blank(),
  panel.grid.minor = element_blank(),
  legend.position = 'right',
  legend.title = element_blank(),
  legend.margin = margin(0, 0, 0, 0),
  legend.box.spacing = unit(0, 'pt'),
  legend.text = element_text(margin = margin(l = 2, r = 0, unit = 'pt')),
  legend.key.size = unit(4, 'mm'),
  axis.text.x = element_text(size = 7, angle = 25, hjust = 0.9),
  strip.text = element_text(size = 8),
  strip.background = element_rect(linetype = 'blank', fill = 'gray95'),
  plot.title = element_text(hjust = 0.5)
)

do_plot_ci <- T
opt_bar_ci <- function(...) {
  if (do_plot_ci) {
    geom_errorbar(...)
  } else {
    geom_blank(...)
  }
}

# internal plot
# dat.trans.classes.counts.rel %>%
#   filter(hasdev) %>%
#   ggplot(aes(faultmodel, n)) +
#   ggtitle('Fault Coverage per FM and Target') +
#   geom_col(aes(fill=fclass)) +
#   opt_bar_ci(data=dat.trans.classes.counts.rel.ci, aes(faultmodel, s, ymin=ymin, ymax=ymax), width = 0, size = 1) +
#   geom_text(data = dat.trans.classes.counts.rel.ci, aes(faultmodel, if(do_plot_ci){ymax + 0.02}else{s + 0.02}, label = sprintf('%.2f', s)), vjust = 0) +
#   scale_fill_discrete(direction = -1) +
#   facet_grid(target ~ .)

message('Plotting to \'plot-faultcoverage-per-fm.pdf\' in current dir...')
# paper plot
# cairo_pdf('plot-faultcoverage-per-fm.pdf', width = 3.1, height = 5, pointsize = 8)
cairo_pdf('plot-faultcoverage-per-fm.pdf', width = 7.0, height = 2, pointsize = 8)
p <- dat.trans.classes.counts.rel %>%
  filter(hasdev) %>%
  ggplot(aes(faultmodel, n)) +
  geom_col(aes(fill = fclass)) +
  opt_bar_ci(
    data = dat.trans.classes.counts.rel.ci,
    aes(faultmodel, s, ymin = ymin, ymax = ymax),
    width = 0.2,
    size = 0.25) +
  geom_text(
    data = dat.trans.classes.counts.rel.ci,
    aes(faultmodel, if(do_plot_ci){ymax + 0.02}else{s + 0.02}, label = sprintf('%.2f', s)),
    size = 2.3,
    vjust = 0) +
  scale_fill_discrete(direction = -1) +
  scale_y_continuous(name = 'Fault Coverage',
                     limits = c(0, 1.05),
                     breaks = seq(0, 1, 0.2)) +
  xlab('Fault Type') +
  facet_grid(~target) +
  paper_theme
print(p)
dev.off()

# dat.trans.classes.counts.rel.target %>%
#   filter(hasdev) %>%
#   ggplot(aes(target, n)) +
#   ggtitle('Fault Coverage per Target') +
#   geom_col(aes(fill=fclass)) +
#   opt_bar_ci(data=dat.trans.classes.counts.rel.target.ci, aes(target, s, ymin=ymin, ymax=ymax), width = 0, size = 1) +
#   scale_fill_discrete(direction = -1)


dat.timmi <- csv_data %>%
  group_by(target) %>%
  summarize(sym_mean = mean(time_sym),
            cmp_mean = mean(time_cmp),
            sym_sd = sd(time_sym),
            cmp_sd = sd(time_cmp),
            sym_sum = sum(time_sym),
            cmp_sum = sum(time_cmp))
dat.timmi.sec <- dat.timmi %>%
  mutate(sym_mean = sym_mean/1e3,
         cmp_mean = cmp_mean/1e3,
         sym_sd = sym_sd/1e3,
         cmp_sd = cmp_sd/1e3,
         sym_sum = sym_sum/1e3/60,
         cmp_sum = cmp_sum/1e3/60)

message('Times in ms...')
print(dat.timmi, n=Inf)
message('Times in sec and hours...')
print(dat.timmi.sec, n=Inf)

# csv_data %>%
#   group_by(target) %>% top_n(-24900, wt = time_sym) %>%
#   ggplot() + 
#   geom_density(aes(x=time_sym), fill='grey70') +
#   geom_vline(data = csv_data %>% group_by(target) %>% mutate(m = mean(time_sym)), aes(xintercept = m)) + 
#   geom_vline(data = csv_data %>% group_by(target) %>% mutate(m = median(time_sym)), aes(xintercept = m), color='blue') + 
#   facet_wrap(target~., scales = 'free')
# csv_data %>%
#   group_by(target) %>% top_n(-24800, wt = time_cmp) %>%
#   ggplot() + 
#   geom_density(aes(x=time_cmp), fill='grey70') +
#   geom_vline(data = csv_data %>% group_by(target) %>% mutate(m = mean(time_cmp)), aes(xintercept = m)) + 
#   geom_vline(data = csv_data %>% group_by(target) %>% mutate(m = median(time_cmp)), aes(xintercept = m), color='blue') + 
#   facet_wrap(target~., scales = 'free')


# dat.timmi.sec %>%
#   ggplot(aes(target, sym_mean, fill=faultmodel)) +
#   geom_col(position = position_dodge())

message('--Fin--')