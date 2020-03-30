#!/usr/bin/env python3
from os import listdir
from os import rename
import logging

import fileinput

def is_zero(line):
    return line.split(',')[2] == '0'

def fix_function_calls(f, outdir, folder):
    logging.info('Fixing function calls')
    stack = []
    mapping = {}
    data = []
    map_res = ''
    g_res = ''
    f_res = ''
    #Uncomment the next section if the llfi_index of call instructions is needed...
    '''
    with open(folder + '/' + f, 'r') as srcfile:
        data = srcfile.readlines()
        
        for line in data:
            if 'call-' in line and '-d' in line:
                if is_zero(line):
                    stack.append(line)
                else:
                    tmp = stack.pop()
                    mapping[tmp] = line
    
    logging.info('Doing the function call replacements...')
    res = ''
    current_tr = 0
    v_set = set(mapping.values())
    k_set = set(mapping.keys())
    for line in data:
        r_tid = line
        if line in k_set:
            res += replaceTimestamp(r_tid, mapping[line])
        if line.startswith('GlobalVariables:'):
            g_res += line.replace('GlobalVariables:', '').lstrip()
        elif line.startswith('Mapping:'):
            map_res += line.replace('Mapping:', '').lstrip()
        elif line.startswith('FAULT:'):
            f_res += line.replace('FAULT:', '').lstrip()
        elif line not in v_set:
            res += r_tid
    '''
   #Comment this section if the llfi_index for call instructions is needed...
    with open(folder + '/' + f, 'r') as srcfile:
        data = srcfile.readlines()
    
    logging.info('Doing the function call replacements...')
    res = ''
    current_tr = 0
    for line in data:
        r_tid = line
        if line.startswith('GlobalVariables:'):
            g_res += line.replace('GlobalVariables:', '').lstrip()
        elif line.startswith('Mapping:'):
            map_res += line.replace('Mapping:', '').lstrip()
        elif line.startswith('FAULT:'):
            f_res += line.replace('FAULT:', '').lstrip()
        elif ',call-' in line and '-d,' in line and not is_zero(line):
            continue
        else:
            res += r_tid

    logging.info('Creating globals file...')
    with open(outdir + '/globals', 'w') as gdata:
        gdata.write(g_res)

    logging.info('Creating thread mapping file...')
    with open(outdir + '/mapping', 'w') as mdata:
        mdata.write(map_res)

    logging.info('Creating fault injection mapping file...')
    with open(outdir + '/faultinj', 'w') as fdata:
        fdata.write(f_res)

    logging.info('Writing the changes to function calls')
    f = f.replace('.txt', '')
    with open(outdir + '/' + f, 'w') as dstfile:
        dstfile.write(res)
    logging.info('Finished fixing function calls')

def replaceTID(src, i):
    res = src.split(',')
    res[1] = res[1] +'_'+ str(i)
    if i != 0:
        print(res)
        print(src)
    return ",".join(res)



def get_mappings(f, outdir, folder):
    logging.info('adding mappings...')
    res = ''
    with open(folder + '/' + f, 'r') as data:
        lines = data.readlines()
        for line in lines:
            if line.startswith('Mapping:'):
                tmp = line.replace('Mapping:', '').lstrip()
                res += tmp
    with open(outdir + '/' + f, 'w') as resfile:
        resfile.write(res)
    logging.info('Finished adding mappings...')

def get_globals(f, outdir, folder):
    logging.info('Adding globals...')
    with open(folder + '/' + f, 'r') as data:
        with open(outdir +'/globals.txt', 'w+') as res:
            for line in data:
                if line.startswith('GlobalVariables:'):
                    tmp = line.replace('GlobalVariables:', '').strip()
                    res.write(tmp + '\n')
                else:
                    break
    logging.info('Finished adding globals...')

def replaceTimestamp(src, dst):
    result = ""
    timestamp = src
    timestamp = timestamp.split(',')[0]
    res = timestamp + "," + ",".join(dst.split(',')[1:])
    tmp1 = res.split(',')
    tmp2 = src.split(',')
    tmp1[3] = tmp2[3]
    return ",".join(tmp1)

def main(folder, outdir, f):
#    get_globals(f, outdir, folder)
    fix_function_calls(f, outdir, folder)

if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO, format='%(asctime)s %(message)s', datefmt='%m/%d/%Y %I:%M:%S %p')
    main('target-programs/parsec/pkgs/apps/swaptions/run', 'target-programs/parsec/pkgs/apps/swaptions/run', 'llfi.stat.trace139800238556928.txt')
