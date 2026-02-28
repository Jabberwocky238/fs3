package main

/*
#include <stdlib.h>
*/
import "C"

import (
	"context"
	"fmt"
	"unsafe"

	corev1 "k8s.io/api/core/v1"
	"k8s.io/apimachinery/pkg/api/errors"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/client-go/kubernetes"
	"k8s.io/client-go/rest"
	"k8s.io/client-go/tools/clientcmd"
)

type ConfigMapClient struct {
	clientSet *kubernetes.Clientset
	namespace string
}

func NewConfigMapClient(namespace string, kubeconfigPath string) (*ConfigMapClient, error) {
	var config *rest.Config
	var err error

	if kubeconfigPath != "" {
		config, err = clientcmd.BuildConfigFromFlags("", kubeconfigPath)
	} else {
		config, err = rest.InClusterConfig()
	}

	if err != nil {
		return nil, fmt.Errorf("failed to load k8s config: %w", err)
	}

	clientSet, err := kubernetes.NewForConfig(config)
	if err != nil {
		return nil, fmt.Errorf("failed to create k8s client: %w", err)
	}

	if namespace == "" {
		namespace = corev1.NamespaceDefault
	}

	return &ConfigMapClient{
		clientSet: clientSet,
		namespace: namespace,
	}, nil
}

func (c *ConfigMapClient) Get(ctx context.Context, name string) (*corev1.ConfigMap, error) {
	return c.clientSet.CoreV1().ConfigMaps(c.namespace).Get(ctx, name, metav1.GetOptions{})
}

func (c *ConfigMapClient) Create(ctx context.Context, name string, data map[string]string) (*corev1.ConfigMap, error) {
	cm := &corev1.ConfigMap{
		ObjectMeta: metav1.ObjectMeta{
			Name:      name,
			Namespace: c.namespace,
		},
		Data: data,
	}

	return c.clientSet.CoreV1().ConfigMaps(c.namespace).Create(ctx, cm, metav1.CreateOptions{})
}

func (c *ConfigMapClient) Update(ctx context.Context, name string, data map[string]string) (*corev1.ConfigMap, error) {
	cm, err := c.Get(ctx, name)
	if err != nil {
		return nil, err
	}

	if cm.Data == nil {
		cm.Data = make(map[string]string)
	}

	for k, v := range data {
		cm.Data[k] = v
	}

	return c.clientSet.CoreV1().ConfigMaps(c.namespace).Update(ctx, cm, metav1.UpdateOptions{})
}

func (c *ConfigMapClient) SetData(ctx context.Context, name string, data map[string]string) (*corev1.ConfigMap, error) {
	cm, err := c.Get(ctx, name)
	if err != nil {
		return nil, err
	}

	cm.Data = data

	return c.clientSet.CoreV1().ConfigMaps(c.namespace).Update(ctx, cm, metav1.UpdateOptions{})
}

var client *ConfigMapClient

//export InitConfigMapClient
func InitConfigMapClient(cNamespace *C.char) *C.char {
	namespace := C.GoString(cNamespace)
	if namespace == "" {
		namespace = "default"
	}
	var err error
	client, err = NewConfigMapClient(namespace, "")
	if err != nil {
		return C.CString(err.Error())
	}
	return nil
}

//export EnsureConfigMapExists
func EnsureConfigMapExists(cName *C.char, cData *C.char) *C.char {
	name := C.GoString(cName)
	data := C.GoString(cData)
	
	_, err := client.Get(context.Background(), name)
	if err == nil {
		return nil
	}
	if !errors.IsNotFound(err) {
		return C.CString(err.Error())
	}
	
	_, err = client.Create(context.Background(), name, map[string]string{"storage.json": data})
	if err != nil {
		return C.CString(err.Error())
	}
	return nil
}

//export ReadConfigMap
func ReadConfigMap(cName *C.char, outData **C.char) *C.char {
	name := C.GoString(cName)
	cm, err := client.Get(context.Background(), name)
	if err != nil {
		return C.CString(err.Error())
	}
	
	val, ok := cm.Data["storage.json"]
	if !ok {
		return C.CString("configmap data.storage.json missing")
	}
	
	*outData = C.CString(val)
	return nil
}

//export WriteConfigMap
func WriteConfigMap(cName *C.char, cData *C.char) *C.char {
	name := C.GoString(cName)
	data := C.GoString(cData)
	
	_, err := client.SetData(context.Background(), name, map[string]string{"storage.json": data})
	if err != nil {
		return C.CString(err.Error())
	}
	return nil
}

//export FreeCString
func FreeCString(cStr *C.char) {
	if cStr != nil {
		C.free(unsafe.Pointer(cStr))
	}
}

func main() {}
