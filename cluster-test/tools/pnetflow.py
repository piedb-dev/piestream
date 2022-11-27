import argparse
import pyshark
import time
import pprint
import sys
import asyncio
import netifaces as ni

import os

debug = False

#
# local information
#
localip = ''

#
# key = "saddr:sport daddr:dport", 
# value = bytes in the past interval
#
ip_flow_stat = {}

#
# key = "src_service dst_service", 
# {bytes = bytes in the past interval
#   ip_flow_stat = {}
# }
#
service_flow_stat = {}

#
# key = "addr:port", "addr:pid"
# value = service name (process)
#
endpoint_service = {}

#
# key = port
# value = service name
#
service_info_whitelist_port = {
    '5507': 'meta',
    '5505': 'frontend',
    '9500': 'prometheus',
    '3000': 'grafana',
    '2181': 'zookeeper',
    '9092': 'kafka',
    '9307': 'minio_api',
    '9400': 'minio_console',
    '9000': 'minio_api_head',
    '9001': 'minio_console_head',
}

service_info_blacklist_port = {
    '22': 'ssh',
}


def get_ip_address(ifname):
    return ni.ifaddresses(ifname)[ni.AF_INET][0]['addr']

def set_parser():
    # Create the parser
    parser = argparse.ArgumentParser()

    # Add an argument
    parser.add_argument('--iface', type=str, required=True)

    return parser

def set_capture(iface):

    # define capture object
    capture = pyshark.LiveCapture(interface=iface)

    print("listening on %s" % iface)

    return capture

def get_service_from_addr_pid(addr, pid):

    if addr == localip:
        # it is local service
        cmd = "sudo ps -ef | awk '{if ($2==" + str(pid) + ") {print $9;}}'"
    elif '122.70.153.' in addr:
        cmd = "ssh "+ str(addr) + " ps -ef | awk '{if ($2==" + str(pid) + ") {print $9;}}'"

    # print('Run cmd', cmd)
    stream = os.popen(cmd)
    output = stream.read().strip()
    return output

def print_service_flow_stat():
    #
    # print service_flow_stat
    #
    for key_sfs in sorted(service_flow_stat.keys()):
        record = service_flow_stat[key_sfs]
        print(key_sfs.ljust(30), str(record['bytes']).rjust(10))
        if False:
            for key_ifs in sorted(record['ip_flow_stat'].keys()):
                print(' '.ljust(60), key_ifs.ljust(30), record['ip_flow_stat'][key_ifs])

def get_service_from_addr_port(addr, port):

    key_service = '{}:{}'.format(addr, port)

    if key_service in endpoint_service.keys():
        return endpoint_service[key_service]
    
    if port in service_info_whitelist_port.keys():
        output = service_info_whitelist_port[port]
        endpoint_service[key_service] = output
        return output

    # obtain service info for this addr:port

    if addr == localip:
        # it is local service
        cmd = 'sudo netstat -anp |grep {addr}:{port} | grep -o " [0-9]*\/[a-z0-9A-Z]* "'.format(
            addr=addr, port=port)
    elif '122.70.153.' in addr:
        cmd = 'ssh {addr} sudo netstat -anp |grep {addr}:{port} | grep -o " [0-9]*\/[a-z0-9A-Z]* "'.format(
            addr=addr, port=port)

    # print('Run cmd', cmd)
    stream = os.popen(cmd)
    output = stream.read().strip()

    # print(":".join("{:02x}".format(ord(c)) for c in output))
    if output != '':
        
        # 
        # output == '3410505/piestream'
        #
        key_service = '{}:{}'.format(addr, output)

        if key_service in endpoint_service.keys():
            return endpoint_service[key_service] 

        if '/' in output:
            # a process that is not pre-defined
            idx = output.find('/')
            pid = output[0:idx]
            service_name = get_service_from_addr_pid(addr, pid).strip()
            
            if service_name != '':
                endpoint_service[key_service] = service_name
                endpoint_service['{}:{}'.format(addr, port)] = service_name
                return service_name
            else:
                print('Error: output =', output, '%s:%s'%(addr, port), 'service_name is empty')

        endpoint_service['{}:{}'.format(addr, port)] = output

    return output

def callback_update_stat(packet):
    # 
    # update statistics in every 5 second
    #
    tot_bytes = 0
   
    try:
        # get timestamp
        localtime = time.asctime(time.localtime(time.time()))
     
        # get packet content
        protocol = packet.transport_layer   # protocol type
        src_addr = packet.ip.src            # source address
        src_port = packet[protocol].srcport   # source port
        dst_addr = packet.ip.dst            # destination address
        dst_port = packet[protocol].dstport   # destination port

        if '122.70.153.' not in src_addr or '122.70.153.' not in dst_addr \
            or src_port in service_info_blacklist_port.keys() \
            or dst_port in service_info_blacklist_port.keys():
            return

        # print(src_addr, src_port, dst_addr, dst_port, packet.length)

        key_stat = '{}:{} ---> {}:{}'.format(src_addr, src_port, dst_addr, dst_port)

        ip_flow_stat.setdefault(key_stat, 0)
        ip_flow_stat[key_stat] += int(packet.length)
        tot_bytes += int(packet.length)

        src_service = get_service_from_addr_port(src_addr, src_port)
        dst_service = get_service_from_addr_port(dst_addr, dst_port)

        # print('service: ', src_service, dst_service)

        if src_service == '' or dst_service == '':
            return

        # print(src_addr, src_port, dst_addr, dst_port, packet.length)
        # print(src_service, dst_service, ip_flow_stat[key_stat])

        key_sfs = '%s ---> %s %s ---> %s'%(src_service.ljust(20), dst_service.ljust(20), src_addr.ljust(20), dst_addr.ljust(20))
        service_flow_stat.setdefault(key_sfs, {})
        service_flow_stat[key_sfs].setdefault('bytes', 0)

        service_flow_stat[key_sfs]['bytes'] += int(packet.length)

        service_flow_stat[key_sfs].setdefault('ip_flow_stat', {key_stat: 0})
        service_flow_stat[key_sfs]['ip_flow_stat'][key_stat] += int(packet.length)

        if debug == True:
            # output packet info
            print (
                src_service.ljust(16), '->', dst_service.ljust(16),
                ('%s:%s'%(src_addr, src_port)).ljust(22), '->', ('%s:%s'%(dst_addr, dst_port)).ljust(22),
                str(packet.length))

    except AttributeError as e:
        # ignore packets other than TCP, UDP and IPv4
        pass

    #print('Total:', tot_bytes)

def do_capture(capture):
    #try:
    #    capture.apply_on_packets(callback_update_stat, timeout=5)
    #except asyncio.TimeoutError as e:
    #    print('Capture ends.')
    
    try:
        capture.sniff(timeout=5)
    except Exception as e:
        pass

    for i in range(len(capture)):
        callback_update_stat(capture[i])

    print_service_flow_stat()


def main():
    global localip

    parser = set_parser()

    # Parse the argument
    args = parser.parse_args()

    localip = get_ip_address(args.iface)

    capture = set_capture(args.iface)

    do_capture(capture)
    
main()